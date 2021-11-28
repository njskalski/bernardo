/*
this widget is supposed to offer:
- tree view on the right, along with scrolling,
- file list view on the most of display (with scrolling as well)
- filename edit box
- buttons save and cancel

I hope I will discover most of functional constraints while implementing it.
 */

use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use log::{debug, warn};

use crate::experiments::focus_group::{FocusGroup, FocusGroupImpl, FocusUpdate};
use crate::experiments::from_geometry::from_geometry;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::layout::cached_sizes::DisplayState;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::theme::Theme;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::button::ButtonWidget;
use crate::widget::edit_box::EditBoxWidget;
use crate::widget::list_widget::ListWidget;
use crate::widget::mock_file_list::mock::{get_mock_file_list, MockFile};
use crate::widget::stupid_tree::{get_stupid_tree, StupidTree};
use crate::widget::tree_view::TreeViewWidget;
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::save_file_dialog::filesystem_provider::FilesystemProvider;

pub struct SaveFileDialogWidget {
    id: WID,

    display_state: Option<DisplayState>,

    tree_widget: TreeViewWidget<PathBuf>,
    list_widget: ListWidget<MockFile>,
    edit_box: EditBoxWidget,

    ok_button: ButtonWidget,
    cancel_button: ButtonWidget,

    // TODO this will probably get moved
    filesystem_provider: Box<dyn FilesystemProvider>,
}

#[derive(Clone, Copy, Debug)]
pub enum SaveFileDialogMsg {
    FocusUpdateMsg(FocusUpdate)
}

impl AnyMsg for SaveFileDialogMsg {}

impl SaveFileDialogWidget {
    pub fn new(filesystem_provider: Box<dyn FilesystemProvider>) -> Self {
        let file_list = get_mock_file_list();
        let tree = filesystem_provider.get_root();
        let tree_widget = TreeViewWidget::<PathBuf>::new(tree);
        let list_widget = ListWidget::new().with_items(file_list).with_selection();
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
            filesystem_provider
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
        if self.display_state.as_ref().map(|x| x.for_size == max_size) == Some(true) {
            return max_size
        }

        let res_sizes = self.todo_internal_layout(max_size);

        debug!("size {}, res_sizes {:?}", max_size, res_sizes);
        self.display_state = Some(DisplayState::new(max_size, res_sizes));

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
                let mut ds: &mut DisplayState = self.display_state.as_mut().unwrap();
                let fg = &mut ds.focus_group;
                let msg = fg.update_focus(fc);
                warn!("focus updated {}", msg);
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

    fn render(&self, theme: &Theme, focused: bool, output: &mut Output) {
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

                    widget.render(theme, focused_op == Some(widget.id()), &mut SubOutput::new(Box::new(output), wir.rect));
                }
            }
        }
    }
}