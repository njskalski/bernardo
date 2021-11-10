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
use std::ops::DerefMut;

use log::warn;

use crate::experiments::focus_group::{FocusGroup, FocusGroupImpl, FocusUpdate};
use crate::experiments::from_geometry::from_geometry;
use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::layout::cached_sizes::CachedSizes;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::button::ButtonWidget;
use crate::widget::edit_box::EditBoxWidget;
use crate::widget::list_widget::ListWidget;
use crate::widget::mock_file_list::mock::{get_mock_file_list, MockFile};
use crate::widget::stupid_tree::{get_stupid_tree, StupidTree};
use crate::widget::tree_view::TreeViewWidget;
use crate::widget::widget::{get_new_widget_id, WID, Widget};

pub struct SaveFileDialogWidget {
    id: WID,

    layout : Box<dyn Layout<Self>>,
    cached_sizes: RefCell<Option<CachedSizes>>,

    tree_widget: TreeViewWidget<usize>,
    list_widget: ListWidget<MockFile>,
    edit_box: EditBoxWidget,

    ok_button: ButtonWidget,
    cancel_button: ButtonWidget,
}

#[derive(Clone, Copy, Debug)]
pub enum SaveFileDialogMsg {
    FocusUpdate(FocusUpdate)
}

impl AnyMsg for SaveFileDialogMsg {}

impl SaveFileDialogWidget {
    pub fn new() -> Self {
        let layout = Box::new(
            SplitLayout::new(SplitDirection::Vertical)
                .with(SplitRule::Proportional(1.0),
                      Box::new(LeafLayout::<SaveFileDialogWidget>::new(
                          Box::new(|s| &s.tree_widget),
                          Box::new(|s| &mut s.tree_widget),
                      )),
                )
                .with(SplitRule::Proportional(4.0),
                      Box::new(SplitLayout::new(SplitDirection::Vertical)
                          .with(SplitRule::Proportional(1.0),
                                Box::new(LeafLayout::<SaveFileDialogWidget>::new(
                                    Box::new(|s| &s.list_widget),
                                    Box::new(|s| &mut s.list_widget),
                                )))
                          .with(SplitRule::Fixed(1),
                                Box::new(LeafLayout::<SaveFileDialogWidget>::new(
                                    Box::new(|s| &s.list_widget),
                                    Box::new(|s| &mut s.list_widget),
                                )))
                      ),
                )
        );

        let file_list = get_mock_file_list();
        let tree = get_stupid_tree();
        let tree_widget = TreeViewWidget::<usize>::new(Box::new(tree));
        let list_widget = ListWidget::new().with_items(file_list);
        let edit_box = EditBoxWidget::new();

        let ok_button = ButtonWidget::new("OK".to_owned());
        let cancel_button = ButtonWidget::new("Cancel".to_owned());

        SaveFileDialogWidget {
            id: get_new_widget_id(),
            layout,
            cached_sizes : RefCell::new(None),
            tree_widget,
            list_widget,
            edit_box,
            ok_button,
            cancel_button,
        }
    }

    fn todo_wid_to_widget(&self, wid : WID) -> Option<&dyn Widget> {
        if self.ok_button.id() == wid {
            return Some(&self.ok_button)
        }
        if self.cancel_button.id() == wid {
            return Some(&self.cancel_button)
        }
        if self.edit_box.id() == wid {
            return Some(&self.edit_box)
        }
        if self.tree_widget.id() == wid {
            return Some(&self.tree_widget)
        }
        if self.list_widget.id() == wid {
            return Some(&self.list_widget)
        }

        None
    }

    fn todo_wid_to_widget_mut(&mut self, wid : WID) -> Option<&mut dyn Widget> {
        if self.ok_button.id() == wid {
            return Some(&mut self.ok_button)
        }
        if self.cancel_button.id() == wid {
            return Some(&mut self.cancel_button)
        }
        if self.edit_box.id() == wid {
            return Some(&mut self.edit_box)
        }
        if self.tree_widget.id() == wid {
            return Some(&mut self.tree_widget)
        }
        if self.list_widget.id() == wid {
            return Some(&mut self.list_widget)
        }

        None
    }
}

impl Widget for SaveFileDialogWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn typename() -> &'static str {
        "SaveFileDialog"
    }

    fn min_size(&self) -> XY {
        self.layout.min_size(self)
    }

    fn layout(&self, max_size: XY) -> XY {
        if self.cached_sizes.borrow().as_ref().map(|x| x.for_size == max_size) == Some(true) {
            return max_size
        }

        let res_sizes = self.layout.calc_sizes(self, max_size);
        self.cached_sizes.swap(&RefCell::new(Some(CachedSizes::new(max_size, res_sizes))));

        max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            InputEvent::KeyInput(key) => match key {
                key if key.is_arrow() => {
                    match key.as_focus_update() {
                        None => {
                            warn!("failed expected cast to FocusUpdate of {}", key);
                            None
                        }
                        Some(event) => Some(Box::new(SaveFileDialogMsg::FocusUpdate(event)))
                    }
                }
                _ => None
            }
            InputEvent::Tick => None
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn get_focused(&self) -> &dyn Widget {
        return match self.cached_sizes.borrow().as_ref() {
            Some(cached_sizes) => {
                let focused_wid = cached_sizes.focus_group.get_focused();
                let widget = self.todo_wid_to_widget(focused_wid);

                match widget {
                    Some(w) => w,
                    None => {
                        warn!("get_focused on {} failed - widget {} not found. Returning self.", focused_wid, self.id());
                        self
                    }
                }
            }
            None => {
                warn!("get_focused on {} failed - no cached_sizes. Returning self.", self.id());
                self
            }
        }
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        let wid_op : Option<WID> = match self.cached_sizes.borrow_mut().as_ref() {
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
            Some(wid) => self.todo_wid_to_widget_mut(wid).unwrap() // TODO problem
        }
    }

    fn render(&self, focused: bool, output: &mut Output) {
        let focused_op = if focused {
            Some(self.get_focused().id())
        } else {
            None
        };

        match self.cached_sizes.borrow().as_ref() {
            None => warn!("failed rendering save_file_dialog without cached_sizes"),
            Some(cached_sizes) => {
                for wir in &cached_sizes.widget_sizes {
                    match self.todo_wid_to_widget(wir.wid) {
                        None => warn!("failed to match WID {} to sub-widget in save_file_dialog {}", wir.wid, self.id()),
                        Some(widget) => {
                            widget.render(focused_op == Some(widget.id()),
                                          &mut SubOutput::new(Box::new(output), wir.rect));
                        }
                    }
                }
            }
        }
    }
}