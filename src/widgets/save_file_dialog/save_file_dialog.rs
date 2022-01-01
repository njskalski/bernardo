/*
this widget is supposed to offer:
- tree view on the right, along with scrolling,
- file list view on the most of display (with scrolling as well)
- filename edit box
- buttons save and cancel

I hope I will discover most of functional constraints while implementing it.
 */

use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::path::PathBuf;

use log::{debug, warn};

use crate::experiments::focus_group::{FocusGroup, FocusUpdate};
use crate::experiments::scroll::Scroll;
use crate::io::filesystem_tree::filesystem_list_item::FilesystemListItem;
use crate::io::filesystem_tree::filesystem_provider::FilesystemProvider;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::layout::cached_sizes::DisplayState;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::theme::Theme;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::list_widget::ListWidget;
use crate::widget::mock_file_list::mock::{get_mock_file_list, MockFile};
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::button::ButtonWidget;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::tree_view::tree_view_node::ChildRc;

pub struct SaveFileDialogWidget {
    id: WID,

    display_state: Option<DisplayState>,

    tree_widget: TreeViewWidget<PathBuf>,
    list_widget: ListWidget<FilesystemListItem>,
    edit_box: EditBoxWidget,

    ok_button: ButtonWidget,
    cancel_button: ButtonWidget,

    curr_display_path: PathBuf,

    tree_scroll : Scroll,

    // TODO this will probably get moved
    filesystem_provider: Box<dyn FilesystemProvider>,
}

#[derive(Clone, Debug)]
pub enum SaveFileDialogMsg {
    FocusUpdateMsg(FocusUpdate),
    Expanded(ChildRc<PathBuf>),
    Highlighted(ChildRc<PathBuf>),
}

impl AnyMsg for SaveFileDialogMsg {}

impl SaveFileDialogWidget {
    pub fn new(filesystem_provider: Box<dyn FilesystemProvider>) -> Self {
        let tree = filesystem_provider.get_root();
        let tree_widget = TreeViewWidget::<PathBuf>::new(tree)
            .with_on_flip_expand(|widget| {
                let (_, item) = widget.get_highlighted();
                Some(Box::new(SaveFileDialogMsg::Expanded(item)))
            })
            .with_on_highlighted_changed(|widget| {
                let (_, item) = widget.get_highlighted();
                Some(Box::new(SaveFileDialogMsg::Highlighted(item)))
            });


        let list_widget = ListWidget::new().with_selection();
        let edit_box = EditBoxWidget::new().with_enabled(true);

        let ok_button = ButtonWidget::new("OK".to_owned());
        let cancel_button = ButtonWidget::new("Cancel".to_owned());

        SaveFileDialogWidget {
            id: get_new_widget_id(),
            display_state: None,
            tree_widget,
            list_widget,
            edit_box,
            ok_button,
            cancel_button,
            curr_display_path: filesystem_provider.get_root().id().clone(),
            filesystem_provider,
            tree_scroll : Scroll::new(XY::new(30, 1000)) //TODO completely arbitrary
        }
    }

    fn todo_wid_to_widget_or_self(&self, wid: WID) -> &dyn Widget {
        if self.ok_button.id() == wid {
            return &self.ok_button
        }
        if self.cancel_button.id() == wid {
            return &self.cancel_button
        }
        if self.edit_box.id() == wid {
            return &self.edit_box
        }
        if self.tree_widget.id() == wid {
            return &self.tree_widget
        }
        if self.list_widget.id() == wid {
            return &self.list_widget
        }

        warn!("todo_wid_to_widget_or_self on {} failed - widget {} not found. Returning self.", wid, self.id());

        self
    }

    fn todo_wid_to_widget_or_self_mut(&mut self, wid: WID) -> &mut dyn Widget {
        if self.ok_button.id() == wid {
            return &mut self.ok_button
        }
        if self.cancel_button.id() == wid {
            return &mut self.cancel_button
        }
        if self.edit_box.id() == wid {
            return &mut self.edit_box
        }
        if self.tree_widget.id() == wid {
            return &mut self.tree_widget
        }
        if self.list_widget.id() == wid {
            return &mut self.list_widget
        }

        warn!("todo_wid_to_widget_mut_or_self on {} failed - widget {} not found. Returning self.", wid, self.id());

        self
    }

    fn todo_internal_layout(&mut self, max_size: XY) -> Vec<WidgetIdRect> {
        let tree_widget = &mut self.tree_widget;
        let list_widget = &mut self.list_widget;
        let edit_box = &mut self.edit_box;

        let mut left_column = LeafLayout::new(tree_widget);

        let mut list = LeafLayout::new(list_widget);
        let mut edit = LeafLayout::new(edit_box);
        let mut right_column = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Proportional(1.0),
                  &mut list)
            .with(SplitRule::Fixed(1),
                  &mut edit,
            );

        let mut layout = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0),
                  &mut left_column)
            .with(SplitRule::Proportional(4.0),
                  &mut right_column,
            );

        let res = layout.calc_sizes(max_size);

        res
    }

    fn todo_filesystem_updated(&mut self) {}
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

    fn layout(&mut self, max_size: XY) -> XY {
        // TODO this lazy relayouting kills resizing on data change.
        // if self.display_state.as_ref().map(|x| x.for_size == max_size) == Some(true) {
        //     return max_size
        // }

        // TODO relayouting destroys focus selection.

        let res_sizes = self.todo_internal_layout(max_size);

        for wir in &res_sizes {
            if wir.wid == self.tree_widget.id() {
                self.tree_scroll.follow_anchor(wir.rect.size, self.tree_widget.anchor());
            }
        }

        debug!("size {}, res_sizes {:?}", max_size, res_sizes);

        // Retention of focus. Not sure if it should be here.
        let focus_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());

        self.display_state = Some(DisplayState::new(max_size, res_sizes));

        // re-setting focus.
        match (focus_op, &mut self.display_state) {
            (Some(focus), Some(ds)) => { ds.focus_group.set_focused(focus); },
            _ => {}
        };

        max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug!("save_file_dialog.on_input {:?}", input_event);

        return match input_event {
            InputEvent::KeyInput(key) => match key {
                key if key.keycode.is_arrow() && key.modifiers.ALT => {
                    debug!("arrow {:?}", key);
                    match key.keycode.as_focus_update() {
                        None => {
                            warn!("failed expected cast to FocusUpdate of {:?}", key);
                            None
                        }
                        Some(event) => {
                            let msg = SaveFileDialogMsg::FocusUpdateMsg(event);
                            debug!("sending {:?}", msg);
                            Some(Box::new(msg))
                        }
                    }
                }
                unknown_key => {
                    debug!("unknown_key {:?}", unknown_key);
                    None
                }
            }
            InputEvent::Tick => None
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("save_file_dialog.update {:?}", msg);

        let our_msg = msg.as_msg::<SaveFileDialogMsg>();
        if our_msg.is_none() {
            warn!("expecetd SaveFileDialogMsg, got {:?}", msg);
            return None;
        }

        return match our_msg.unwrap() {
            SaveFileDialogMsg::FocusUpdateMsg(focus_update) => {
                warn!("updating focus");
                let fc = *focus_update;
                let ds: &mut DisplayState = self.display_state.as_mut().unwrap();
                let fg = &mut ds.focus_group;
                let msg = fg.update_focus(fc);
                warn!("focus updated {}", msg);
                None
            }
            SaveFileDialogMsg::Expanded(child) => {
                // TODO this looks like shit
                self.filesystem_provider.expand(child.id().as_path());
                self.curr_display_path = child.id().clone();
                let mut items = self.filesystem_provider.get_files(self.curr_display_path.borrow()).collect::<Vec<_>>();
                self.list_widget.set_items(&mut items);

                None
            }
            SaveFileDialogMsg::Highlighted(child) => {
                let items = self.filesystem_provider.get_files(child.id().as_path());
                self.list_widget.set_items_it(items);

                None
            }
            unknown_msg => {
                warn!("SaveFileDialog.update : unknown message {:?}", unknown_msg);
                None
            }
        };
    }

    fn get_focused(&self) -> &dyn Widget {
        return match self.display_state.borrow().as_ref() {
            Some(cached_sizes) => {
                let focused_wid = cached_sizes.focus_group.get_focused();
                self.todo_wid_to_widget_or_self(focused_wid)
            }
            None => {
                warn!("get_focused on {} failed - no cached_sizes. Returning self.", self.id());
                self
            }
        }
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        let wid_op: Option<WID> = match &self.display_state {
            Some(cached_sizes) => {
                Some(cached_sizes.focus_group.get_focused())
            }
            None => {
                warn!("get_focused on {} failed - no cached_sizes.", self.id());
                None
            }
        };

        return match wid_op {
            None => self,
            Some(wid) => self.todo_wid_to_widget_or_self_mut(wid)
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let focused_op = if focused {
            Some(self.get_focused().id())
        } else {
            None
        };

        match self.display_state.borrow().as_ref() {
            None => warn!("failed rendering save_file_dialog without cached_sizes"),
            Some(cached_sizes) => {
                debug!("widget_sizes : {:?}", cached_sizes.widget_sizes);
                for wir in &cached_sizes.widget_sizes {
                    let widget = self.todo_wid_to_widget_or_self(wir.wid);

                    if widget.id() == self.id() {
                        warn!("render: failed to match WID {} to sub-widget in save_file_dialog {}", wir.wid, self.id());
                        continue;
                    }

                    let mut sub_output = &mut SubOutput::new(Box::new(output), wir.rect);

                    if widget.id() != self.tree_widget.id() {
                        widget.render(theme, focused_op == Some(widget.id()), sub_output);
                    }

                    if widget.id() == self.tree_widget.id() {
                        self.tree_scroll.render_within(sub_output, &self.tree_widget, theme, focused_op == Some(widget.id()));
                    }
                }
            }
        }
    }
}