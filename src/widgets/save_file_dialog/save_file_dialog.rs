/*
this widget is supposed to offer:
- tree view on the right, along with scrolling,
- file list view on the most of display (with scrolling as well)
- filename edit box
- buttons save and cancel

I hope I will discover most of functional constraints while implementing it.
 */

use std::borrow::Borrow;
use std::fmt::Debug;
use std::io::Error;
use std::path;
use std::path::{Path, PathBuf, StripPrefixError};
use std::rc::Rc;

use log::{debug, error, warn};

use crate::{FsfRef, Keycode};
use crate::experiments::focus_group::{FocusGroup, FocusUpdate};
use crate::io::filesystem_tree::file_front::{FileFront, FilteredFileFront};
use crate::io::filesystem_tree::filesystem_front::{FilesystemFront, SomethingToSave};
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::layout::display_state::DisplayState;
use crate::layout::empty_layout::EmptyLayout;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::helpers::fill_output;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::theme::Theme;
use crate::primitives::xy::XY;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::widget::{get_new_widget_id, WID, Widget, WidgetAction, WidgetActionParam};
use crate::widgets::button::ButtonWidget;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::generic_dialog::generic_dialog::GenericDialog;
use crate::widgets::list_widget::ListWidget;
use crate::widgets::save_file_dialog::dialogs::override_dialog;
use crate::widgets::save_file_dialog::save_file_dialog::SaveFileDialogMsg::Save;
use crate::widgets::save_file_dialog::save_file_dialog_msg::SaveFileDialogMsg;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::tree_view::tree_view_node::{MaybeBool, TreeViewNode};
use crate::widgets::with_scroll::WithScroll;

// TODO now it displays both files and directories in tree view, it should only directories


const OK_LABEL: &'static str = "OK";
const CANCEL_LABEL: &'static str = "CANCEL";


pub struct SaveFileDialogWidget {
    id: WID,

    display_state: Option<DisplayState>,

    tree_widget: WithScroll<TreeViewWidget<PathBuf, Rc<FileFront>>>,
    list_widget: ListWidget<Rc<FileFront>>,
    edit_box: EditBoxWidget,

    ok_button: ButtonWidget,
    cancel_button: ButtonWidget,

    fsf: FsfRef,

    on_cancel: Option<WidgetAction<Self>>,
    on_save: Option<WidgetAction<Self>>,
    on_save_fail: Option<WidgetActionParam<Self, Option<std::io::Error>>>,

    root_path: PathBuf,

    to_save: Option<Box<dyn SomethingToSave>>,

    hover_dialog: Option<GenericDialog>,
}

impl SaveFileDialogWidget {
    pub fn new(fsf: FsfRef) -> Self {
        let tree = fsf.get_root();
        // let just_dirs = FilteredFileFront::new(tree.clone(), |i| {
        //     i.is_dir()
        // });
        //

        let filter = |f: &Rc<FileFront>| -> bool {
            f.is_dir()
        };

        let tree_widget = TreeViewWidget::<PathBuf, Rc<FileFront>>::new(tree)
            .with_filter(filter, Some(0))
            .with_on_flip_expand(|widget| {
                let (_, item) = widget.get_highlighted();
                Some(Box::new(SaveFileDialogMsg::TreeExpanded(item)))
            })
            .with_on_highlighted_changed(|widget| {
                let (_, item) = widget.get_highlighted();
                Some(Box::new(SaveFileDialogMsg::TreeHighlighted(item)))
            });

        let scroll_tree_widget = WithScroll::new(tree_widget, ScrollDirection::Vertical);

        let list_widget: ListWidget<Rc<FileFront>> = ListWidget::new().with_selection()
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

        let path = fsf.get_root().path().to_owned();

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
            on_save_fail: None,
            on_cancel: None,
            root_path: path,
            to_save: None,
            hover_dialog: None,
        }
    }

    pub fn with_something_to_save(self, to_save: Box<dyn SomethingToSave>) -> Self {
        Self {
            to_save: Some(to_save),
            ..self
        }
    }

    pub fn set_something_to_save(&mut self, to_save: Option<Box<dyn SomethingToSave>>) {
        self.to_save = to_save;
    }

    pub fn with_on_save_fail(self, on_save_fail: WidgetActionParam<Self, Option<std::io::Error>>) -> Self {
        Self {
            on_save_fail: Some(on_save_fail),
            ..self
        }
    }

    pub fn set_on_save_fail(&mut self, on_save_fail_op: Option<WidgetActionParam<Self, Option<std::io::Error>>>) {
        self.on_save_fail = on_save_fail_op;
    }

    fn internal_layout(&mut self, max_size: XY) -> Vec<WidgetIdRect> {
        let tree_widget = &mut self.tree_widget;
        let list_widget = &mut self.list_widget;
        let edit_box = &mut self.edit_box;

        let mut left_column = LeafLayout::new(tree_widget);
        let mut empty = EmptyLayout::new();
        let mut ok_box = LeafLayout::new(&mut self.ok_button);
        let mut cancel_box = LeafLayout::new(&mut self.cancel_button);

        let mut button_bar = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0), &mut empty)
            .with(SplitRule::Fixed(12), &mut cancel_box)
            .with(SplitRule::Fixed(12), &mut ok_box);


        let mut list = LeafLayout::new(list_widget);
        let mut edit = LeafLayout::new(edit_box);
        let mut right_column = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Proportional(1.0),
                  &mut list)
            .with(SplitRule::Fixed(1),
                  &mut edit)
            .with(SplitRule::Fixed(1),
                  &mut button_bar);

        let mut layout = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0),
                  &mut left_column)
            .with(SplitRule::Proportional(4.0),
                  &mut right_column,
            );


        match &mut self.hover_dialog {
            None => layout.calc_sizes(max_size),
            Some(dialog) => {
                let size = dialog.min_size() * 3;
                let mut leaf_dialog = LeafLayout::new(dialog);
                let mut layout = &mut HoverLayout::new(&mut layout,
                                                       &mut leaf_dialog,
                                                       Rect::new(
                                                           XY::new(6, 3), // TODO
                                                           size,
                                                       ));
                layout.calc_sizes(max_size)
            }
        }
    }


    pub fn set_path(&mut self, path: &Path) -> bool {
        if !self.fsf.is_dir(path) {
            warn!("attempted to set path to non-dir: {:?}", path);
            return false;
        }

        let mut root_path = self.fsf.get_root().path().to_owned();

        match path.strip_prefix(&root_path) {
            Err(e) => {
                error!("supposed to set path to {:?}, but it's outside fs {:?}, because: {}", path, &root_path, e);
            }
            Ok(remainder) => {
                for comp in remainder.components() {
                    self.fsf.todo_expand(&root_path);
                    debug!("expanding subtree {:?}", root_path);

                    // TODO one can save some memory here
                    self.tree_widget.internal_mut().expanded_mut().insert(root_path.clone());

                    root_path = root_path.join(comp);
                }
            }
        }

        self.root_path = path.to_owned();

        true
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

    pub fn set_on_save(&mut self, on_save: WidgetAction<Self>) {
        self.on_save = Some(on_save);
    }

    pub fn with_on_save(self, on_save: WidgetAction<Self>) -> Self {
        Self {
            on_save: Some(on_save),
            ..self
        }
    }

    pub fn get_path(&self) -> Option<PathBuf> {
        if self.edit_box.get_text().len_chars() == 0 {
            None
        } else {
            let path = self.tree_widget.internal().get_highlighted().1.path().to_owned();
            let last_item = self.edit_box.get_text().to_string();
            Some(path.join(last_item))
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
                return self.on_save_fail.map(|handle| handle(self, None)).flatten()
            }
        };

        warn!("will save to {:?}", path);

        if self.fsf.exists(&path) {
            let filename = self.edit_box.get_text().to_string();

            self.hover_dialog = Some(override_dialog(filename));
            return None;
        } else {
            self.save_positively()
        }
    }

    // Returns op message to parent, so it can be called from 'update'
    fn save_positively(&self) -> Option<Box<dyn AnyMsg>> {
        let bytes: Vec<u8> = match &self.to_save {
            Some(sts) => sts.get_bytes().map(|i| *i).collect(),
            None => {
                error!("nothing to save");
                return self.on_save_fail.map(|handler| handler(self, None)).flatten()
            }
        };

        let path = match self.get_path() {
            Some(p) => p,
            None => {
                error!("can't save of empty path. The OK button should have been blocked!");
                return self.on_save_fail.map(|handler| handler(self, None)).flatten()
            }
        };

        match self.fsf.todo_save_file_sync(path.as_path(), &bytes) {
            Ok(_) => {
                self.on_save.map(|handler| handler(self)).flatten()
            }
            Err(e) => {
                error!("save error: {}", e);
                self.on_save_fail.map(|handler| handler(self, Some(e))).flatten()
            }
        }
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
        let max_size = sc.hint().size;

        // TODO this lazy relayouting kills resizing on data change.
        // if self.display_state.as_ref().map(|x| x.for_size == max_size) == Some(true) {
        //     return max_size
        // }

        // TODO relayouting destroys focus selection.

        let res_sizes = self.internal_layout(max_size);


        // Retention of focus. Not sure if it should be here.
        let focus_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());

        let mut ds = DisplayState::new(max_size, res_sizes);
        ds.focus_group_mut().add_edge(self.tree_widget.id(), FocusUpdate::Right, self.list_widget.id());
        ds.focus_group_mut().add_edge(self.list_widget.id(), FocusUpdate::Left, self.tree_widget.id());

        ds.focus_group_mut().add_edge(self.edit_box.id(), FocusUpdate::Left, self.tree_widget.id());

        ds.focus_group_mut().add_edge(self.edit_box.id(), FocusUpdate::Up, self.list_widget.id());
        ds.focus_group_mut().add_edge(self.list_widget.id(), FocusUpdate::Down, self.edit_box.id());


        // debug!("focusgroup: {:?}", ds.focus_group);

        self.display_state = Some(ds);


        // re-setting focus.
        match (focus_op, &mut self.display_state) {
            (Some(focus), Some(ds)) => { ds.focus_group.set_focused(focus); },
            _ => {}
        };

        max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        // debug!("save_file_dialog.on_input {:?}", input_event);

        return match input_event {
            InputEvent::FocusUpdate(focus_update) => {
                Some(Box::new(SaveFileDialogMsg::FocusUpdateMsg(focus_update)))
            },
            InputEvent::KeyInput(key) => {
                match key.keycode {
                    Keycode::Esc => SaveFileDialogMsg::Cancel.someboxed(),
                    _ => None
                }
            }
            _ => None,
        }
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
                warn!("updating focus");
                let fc = *focus_update;
                self.display_state.as_mut().map(
                    |mut ds| {
                        let msg = ds.focus_group.update_focus(*focus_update);
                        warn!("focus updated {}", msg);
                        None
                    }
                ).unwrap_or_else(|| {
                    error!("failed retrieving display_state");
                    None
                })
            }
            SaveFileDialogMsg::TreeExpanded(node) => {
                // TODO load data if necessary
                self.fsf.todo_expand(node.path());

                None
            }
            SaveFileDialogMsg::TreeHighlighted(node) => {
                self.fsf.todo_expand(node.path());
                self.list_widget.set_provider(
                    Box::new(FilteredFileFront::new(node.clone(),
                                                    |f| f.is_file(),
                    ))
                );

                None
            }
            SaveFileDialogMsg::FileListHit(file) => {
                let text = file.path().file_name().map(|f| f.to_str().unwrap_or("error")).unwrap();
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

        match self.display_state.borrow().as_ref() {
            None => warn!("failed rendering save_file_dialog without cached_sizes"),
            Some(cached_sizes) => {
                // debug!("widget_sizes : {:?}", cached_sizes.widget_sizes);
                for wir in &cached_sizes.widget_sizes {
                    match self.get_subwidget(wir.wid) {
                        Some(widget) => {
                            let sub_output = &mut SubOutput::new(output, wir.rect);
                            widget.render(theme,
                                          focused && cached_sizes.focus_group.get_focused() == widget.id(),
                                          sub_output,
                            );
                        },
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