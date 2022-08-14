/*
this widget is supposed to offer:
- tree view on the right, along with scrolling,
- file list view on the most of display (with scrolling as well)
- filename edit box
- buttons save and cancel

I hope I will discover most of functional constraints while implementing it.
 */

use std::borrow::Borrow;
use std::path::PathBuf;
use std::sync::Arc;

use jsonrpc_core::futures::future::{err, ok};
use log::{debug, error, warn};

use crate::{Keycode, subwidget};
use crate::config::theme::Theme;
use crate::experiments::focus_group::FocusUpdate;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::path::SPath;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::layout::display_state::GenericDisplayState;
use crate::layout::empty_layout::EmptyLayout;
use crate::layout::frame_layout::FrameLayout;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::border::SINGLE_BORDER_STYLE;
use crate::primitives::helpers::fill_output;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::widget::{get_new_widget_id, WID, Widget, WidgetAction, WidgetActionParam};
use crate::widgets::button::ButtonWidget;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::generic_dialog::generic_dialog::GenericDialog;
use crate::widgets::list_widget::ListWidget;
use crate::widgets::save_file_dialog::dialogs::override_dialog;
use crate::widgets::save_file_dialog::save_file_dialog_msg::SaveFileDialogMsg;
use crate::widgets::spath_tree_view_node::DirTreeNode;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::with_scroll::WithScroll;

// TODO now it displays both files and directories in tree view, it should only directories

const OK_LABEL: &'static str = "OK";
const CANCEL_LABEL: &'static str = "CANCEL";

pub struct SaveFileDialogWidget {
    id: WID,

    display_state: Option<GenericDisplayState>,

    // TODO at this point I just want it to work, I should profile if it behaves fast.
    tree_widget: WithScroll<TreeViewWidget<SPath, DirTreeNode>>,
    list_widget: ListWidget<SPath>,
    edit_box: EditBoxWidget,

    ok_button: ButtonWidget,
    cancel_button: ButtonWidget,

    fsf: FsfRef,

    on_cancel: Option<WidgetAction<Self>>,
    on_save: Option<WidgetActionParam<Self, SPath>>,

    root_path: SPath,

    hover_dialog: Option<GenericDialog>,
}

impl SaveFileDialogWidget {
    pub fn new(fsf: FsfRef) -> Self {
        let root = fsf.root();

        let tree_widget = TreeViewWidget::<SPath, DirTreeNode>::new(DirTreeNode::new(root.clone()))
            // .with_filter(filter, Some(0))
            .with_on_flip_expand(|widget| {
                let (_, item) = widget.get_highlighted();
                Some(Box::new(SaveFileDialogMsg::TreeExpanded(item.spath().clone())))
            })
            .with_on_highlighted_changed(|widget| {
                let (_, item) = widget.get_highlighted();
                Some(Box::new(SaveFileDialogMsg::TreeHighlighted(item.spath().clone())))
            });

        let scroll_tree_widget = WithScroll::new(tree_widget, ScrollDirection::Vertical);

        let list_widget: ListWidget<SPath> = ListWidget::new().with_selection()
            .with_on_hit(|w| {
                w.get_highlighted().map(|item| {
                    Some(SaveFileDialogMsg::FileListHit(item).boxed())
                }).flatten()
            });
        let edit_box = EditBoxWidget::new().with_enabled(true).with_on_hit(
            |_| SaveFileDialogMsg::EditBoxHit.someboxed()
        );
        let ok_button = ButtonWidget::new(Box::new(OK_LABEL)).with_on_hit(
            |_| SaveFileDialogMsg::Save.someboxed()
        );
        let cancel_button = ButtonWidget::new(Box::new(CANCEL_LABEL)).with_on_hit(
            |_| SaveFileDialogMsg::Cancel.someboxed()
        );

        let path = fsf.root();

        SaveFileDialogWidget {
            id: get_new_widget_id(),
            display_state: None,
            tree_widget: scroll_tree_widget,
            list_widget,
            edit_box,
            ok_button,
            cancel_button,
            fsf,
            on_save: None,
            on_cancel: None,
            root_path: path,
            hover_dialog: None,
        }
    }

    fn internal_layout(&mut self, max_size: XY) -> Vec<WidgetIdRect> {
        let tree_layout = LeafLayout::new(subwidget!(Self.tree_widget));
        // let mut empty_layout = EmptyLayout::new().with_size(XY::new(1, 1));

        let left_column = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Proportional(1.0), tree_layout.boxed())
            // .with(SplitRule::Fixed(1), &mut empty_layout)
            .boxed();

        let ok_box = LeafLayout::new(subwidget!(Self.ok_button)).boxed();
        let cancel_box = LeafLayout::new(subwidget!(Self.cancel_button)).boxed();

        let button_bar = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0), EmptyLayout::new().boxed())
            .with(SplitRule::Fixed(12), cancel_box)
            .with(SplitRule::Fixed(12), ok_box)
            .boxed();

        let list = LeafLayout::new(subwidget!(Self.list_widget)).boxed();
        let edit = LeafLayout::new(subwidget!(Self.edit_box)).boxed();
        let right_column = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Proportional(1.0),
                  list)
            .with(SplitRule::Fixed(1),
                  edit)
            .with(SplitRule::Fixed(1),
                  button_bar)
            .boxed();

        let layout = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(2.0),
                  left_column)
            .with(SplitRule::Proportional(3.0),
                  right_column,
            )
            .boxed();

        let frame = XY::new(1, 1);

        match self.hover_dialog.as_mut() {
            None => FrameLayout::new(layout, frame).calc_sizes(self, max_size),
            Some(dialog) => {
                let margins = max_size / 20;
                //TODO(subwidgetpointermap)
                let dialog_layout = LeafLayout::new(SubwidgetPointer::new(
                    Box::new(|x: &Self| { x.hover_dialog.as_ref().unwrap() }),
                    Box::new(|x: &mut Self| { x.hover_dialog.as_mut().unwrap() }),
                )).boxed();

                FrameLayout::new(HoverLayout::new(layout,
                                                  dialog_layout,
                                                  Rect::new(
                                                      margins, // TODO
                                                      max_size - margins * 2,
                                                  ),
                ).boxed(), frame).calc_sizes(self, max_size)
            }
        }
    }

    /*
    Sets the path of expanded nodes in save file dialog, and potentially filename.
    If set to a dir, only tree will be expanded.
    If set to a file, both tree will be expanded, and editbox filled.
     */
    pub fn set_path(&mut self, path: SPath) -> bool {
        debug!("setting path to {:?}", path);

        let (dir, file): (Option<SPath>, Option<SPath>) = if path.is_file() {
            (path.parent(), Some(path))
        } else {
            (Some(path), None)
        };

        let mut changed = false;
        dir.map(|p| changed = self.tree_widget.internal_mut().expand_path(&p));
        file.map(|p| {
            self.edit_box.set_text(p.label().as_ref());
            changed = true;
        });

        changed
    }


    // fn show_files_on_right_panel(&mut self, directory: &Arc<PathBuf>) -> bool {
    //     // if !self.fsf.is_dir(&directory) {
    //     //     warn!("expected directory, got {:?}", directory);
    //     //     return false;
    //     // }
    //     //
    //     // let item = match self.fsf.get_item(directory) {
    //     //     Some(i) => i,
    //     //     None => {
    //     //         warn!("failed retrieving {:?} from fsf", directory);
    //     //         return false;
    //     //     }
    //     // };
    //     //
    //     // // self.list_widget.set_provider(
    //     // //     Box::new(FilteredSPath::new(item, |f| f.is_file()))
    //     // // );
    //     //
    //     // true
    // }

    pub fn with_path(mut self, path: SPath) -> Self {
        self.set_path(path);
        self
    }

    pub fn set_on_cancel(&mut self, on_cancel: WidgetAction<Self>) {
        self.on_cancel = Some(on_cancel);
    }

    pub fn with_on_cancel(self, on_cancel: WidgetAction<Self>) -> Self {
        Self {
            on_cancel: Some(on_cancel),
            ..self
        }
    }

    pub fn set_on_save(&mut self, on_save: WidgetActionParam<Self, SPath>) {
        self.on_save = Some(on_save);
    }

    pub fn with_on_save(self, on_save: WidgetActionParam<Self, SPath>) -> Self {
        Self {
            on_save: Some(on_save),
            ..self
        }
    }

    // TODO tests!
    pub fn get_path(&self) -> Option<SPath> {
        if self.edit_box.get_text().len_chars() == 0 {
            None
        } else {
            let file_name = self.edit_box.get_text().to_string();
            let path = self.tree_widget.internal().get_highlighted().1.spath().clone();
            let result = path.descendant_unchecked(file_name);

            if result.is_none() {
                error!("expected non-empty result");
            }

            result
        }
    }

    // Returns op message to parent, so it can be called from 'update'
    pub fn save_or_ask_for_override(&mut self) -> Option<Box<dyn AnyMsg>> {
        if self.hover_dialog.is_some() {
            error!("save_or_ask_for_override called from unexpected state");
        }

        let path = match self.get_path() {
            Some(p) => p,
            None => {
                error!("can't save of empty path. The OK button should have been blocked!");
                return None;
            }
        };

        if path.exists() {
            let filename = self.edit_box.get_text().to_string();

            self.hover_dialog = Some(override_dialog(filename));
            return None;
        } else {
            self.save_positively()
        }
    }

    // Returns op message to parent, so it can be called from 'update'
    fn save_positively(&self) -> Option<Box<dyn AnyMsg>> {
        let path = match self.get_path() {
            Some(p) => p,
            None => {
                error!("self.get_path() is None in save_positively!");
                return None
            }
        };

        self.on_save.map(|on_save| {
            on_save(self, path)
        }).unwrap_or_else(|| {
            error!("attempted to save, but on_save not set");
            None
        })
    }
}

impl Widget for SaveFileDialogWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        "SaveFileDialog"
    }

    fn min_size(&self) -> XY {
        XY::new(4, 4)
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        // TODO this entire function is a makeshift and experiment
        let max_size = sc.visible_hint().size;

        // TODO this lazy relayouting kills resizing on data change.
        // if self.display_state.as_ref().map(|x| x.for_size == max_size) == Some(true) {
        //     return max_size
        // }

        // TODO relayouting destroys focus selection.

        let res_sizes = self.internal_layout(max_size);


        // Retention of focus. Not sure if it should be here.
        let focus_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());

        let mut ds = GenericDisplayState::new(max_size, res_sizes);
        ds.focus_group_mut().add_edge(self.tree_widget.id(), FocusUpdate::Right, self.list_widget.id());
        ds.focus_group_mut().add_edge(self.list_widget.id(), FocusUpdate::Left, self.tree_widget.id());

        ds.focus_group_mut().add_edge(self.edit_box.id(), FocusUpdate::Left, self.tree_widget.id());

        ds.focus_group_mut().add_edge(self.edit_box.id(), FocusUpdate::Up, self.list_widget.id());
        ds.focus_group_mut().add_edge(self.list_widget.id(), FocusUpdate::Down, self.edit_box.id());


        // debug!("focusgroup: {:?}", ds.focus_group);

        self.display_state = Some(ds);


        // re-setting focus.
        match (focus_op, &mut self.display_state) {
            (Some(focus), Some(ds)) => { ds.focus_group.set_focused(focus); }
            _ => {}
        };

        max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        // debug!("save_file_dialog.on_input {:?}", input_event);

        return match input_event {
            InputEvent::FocusUpdate(focus_update) => {
                let can_update = self.display_state.as_ref().map(|ds| {
                    ds.focus_group().can_update_focus(focus_update)
                }).unwrap_or(false);

                if can_update {
                    Some(Box::new(SaveFileDialogMsg::FocusUpdateMsg(focus_update)))
                } else {
                    None
                }
            }
            InputEvent::KeyInput(key) => {
                match key.keycode {
                    Keycode::Esc => SaveFileDialogMsg::Cancel.someboxed(),
                    keycode if keycode.is_arrow() => {
                        if let (Some(msg), Some(ds)) = (key.as_focus_update(), &self.display_state) {
                            if ds.focus_group.can_update_focus(msg) {
                                SaveFileDialogMsg::FocusUpdateMsg(msg).someboxed()
                            } else {
                                None
                            }
                        } else {
                            error!("failed to cast arrow to focus update");
                            None
                        }
                    }
                    _ => None
                }
            }
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        // debug!("save_file_dialog.update {:?}", msg);

        let our_msg = msg.as_msg::<SaveFileDialogMsg>();
        if our_msg.is_none() {
            warn!("expecetd SaveFileDialogMsg, got {:?}", msg);
            return None;
        }

        return match our_msg.unwrap() {
            SaveFileDialogMsg::FocusUpdateMsg(focus_update) => {
                // debug!("updating focus");
                self.display_state.as_mut().map(
                    |ds| {
                        if !ds.focus_group.update_focus(*focus_update) {
                            warn!("focus update accepted but failed");
                        }
                        None
                    }
                ).unwrap_or_else(|| {
                    error!("failed retrieving display_state");
                    None
                })
            }
            SaveFileDialogMsg::TreeExpanded(..) => {
                None
            }
            SaveFileDialogMsg::TreeHighlighted(node) => {
                if !self.set_path(node.clone()) {
                    warn!("failed to set path {:?}", node);
                }

                None
            }
            SaveFileDialogMsg::FileListHit(file) => {
                let text = file.file_name_str().unwrap();
                self.edit_box.set_text(text); // TODO
                self.edit_box.set_cursor_end();
                self.set_focused(self.edit_box.id());
                None
            }
            SaveFileDialogMsg::EditBoxHit => {
                self.set_focused(self.ok_button.id());
                None
            }
            SaveFileDialogMsg::Cancel => {
                self.on_cancel.map(|action| action(self)).flatten()
            }
            SaveFileDialogMsg::Save => {
                self.save_or_ask_for_override()
            }
            SaveFileDialogMsg::CancelOverride => {
                self.hover_dialog = None;
                None
            }
            SaveFileDialogMsg::CancelOverride => {
                self.hover_dialog = None;
                self.save_positively()
            }
            unknown_msg => {
                warn!("SaveFileDialog.update : unknown message {:?}", unknown_msg);
                None
            }
        };
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        if self.hover_dialog.is_some() {
            return self.hover_dialog.as_ref().map(|f| f as &dyn Widget);
        };

        let wid_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());
        wid_op.map(|wid| self.get_subwidget(wid)).flatten()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        if self.hover_dialog.is_some() {
            return self.hover_dialog.as_mut().map(|f| f as &mut dyn Widget);
        }

        let wid_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());
        wid_op.map(move |wid| self.get_subwidget_mut(wid)).flatten()
    }

    fn set_focused(&mut self, wid: WID) -> bool {
        if self.hover_dialog.is_some() {
            warn!("blocking setting focus, hovering dialog displayed");
            return false;
        }
        self.display_state.as_mut().map(|ds| {
            ds.focus_group_mut().set_focused(wid)
        }).unwrap_or(false)
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        fill_output(theme.ui.non_focused.background, output);

        SINGLE_BORDER_STYLE.draw_edges(theme.default_text(focused),
                                       output);

        match self.display_state.borrow().as_ref() {
            None => warn!("failed rendering save_file_dialog without cached_sizes"),
            Some(cached_sizes) => {
                // debug!("widget_sizes : {:?}", cached_sizes.widget_sizes);
                let focused_child_id_op = self.get_focused().map(|f| f.id());
                for wir in &cached_sizes.widget_sizes {
                    match self.get_subwidget(wir.wid) {
                        Some(widget) => {
                            let sub_output = &mut SubOutput::new(output, wir.rect);
                            widget.render(theme,
                                          focused && focused_child_id_op == Some(wir.wid),
                                          sub_output,
                            );
                        }
                        None => {
                            warn!("subwidget {} not found!", wir.wid);
                        }
                    }
                }
            }
        }
    }

    fn subwidgets_mut(&mut self) -> Box<dyn std::iter::Iterator<Item=&mut dyn Widget> + '_> {
        // debug!("call to save_file_dialog subwidget_mut on {}", self.id());
        let mut widgets = vec![&mut self.tree_widget as &mut dyn Widget,
                               &mut self.list_widget,
                               &mut self.edit_box,
                               &mut self.ok_button,
                               &mut self.cancel_button];

        self.hover_dialog.as_mut().map(|f| widgets.push(f));

        Box::new(widgets.into_iter())
    }

    fn subwidgets(&self) -> Box<dyn std::iter::Iterator<Item=&dyn Widget> + '_> {
        // debug!("call to save_file_dialog subwidget on {}", self.id());
        let mut widgets = vec![&self.tree_widget as &dyn Widget,
                               &self.list_widget,
                               &self.edit_box,
                               &self.ok_button,
                               &self.cancel_button];

        self.hover_dialog.as_ref().map(|f| widgets.push(f));
        Box::new(widgets.into_iter())
    }
}