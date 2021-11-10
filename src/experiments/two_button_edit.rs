/*
this is experimental widget, which has two buttons (OK and Cancel) and TextBox
and OK is enabled ONLY if text TextBox content length > 4.

just an experiment to see if the design works.
 */

use std::any::Any;
use std::borrow::{Borrow, BorrowMut};
use std::fs::read;
use std::ops::Deref;

use log::warn;

// use crate::layout::split_layout::{SplitLayout, SplitDirection, SplitRule};
use crate::experiments::focus_group::{FocusGroup, FocusGroupImpl, FocusUpdate};
// use crate::layout::fixed_layout::{FixedLayout, FixedItem};
use crate::experiments::from_geometry::from_geometry;
use crate::experiments::two_button_edit::TwoButtonEditMsg::{
    Cancel, FocusUpdateMsg, TextInvalid, TextValid,
};
use crate::experiments::util::default_key_to_focus_update;
use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::layout::cached_sizes::CachedSizes;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::button::ButtonWidget;
use crate::widget::edit_box::EditBoxWidget;
use crate::widget::widget::{get_new_widget_id, WID, Widget};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TwoButtonEditMsg {
    OK,
    Cancel,
    TextValid,
    TextInvalid,
    FocusUpdateMsg(FocusUpdate),
}

impl AnyMsg for TwoButtonEditMsg {}

pub struct TwoButtonEdit {
    id: usize,
    ok_button: ButtonWidget,
    cancel_button: ButtonWidget,
    edit_box: EditBoxWidget,
    layout: Box<dyn Layout<TwoButtonEdit>>,
    cached_sizes : Option<CachedSizes>,
    focus_group: FocusGroupImpl,
}

impl TwoButtonEdit {
    pub fn new() -> Self {
        let ok_button = ButtonWidget::new("OK".into())
            .with_on_hit(|_| Some(Box::new(TwoButtonEditMsg::OK)))
            .with_enabled(false);

        let cancel_button = ButtonWidget::new("Cancel".into())
            .with_on_hit(|_| Some(Box::new(TwoButtonEditMsg::Cancel)));

        let edit_box = EditBoxWidget::new()
            .with_on_change(|eb| {
                if eb.get_text().len() > 4 {
                    Some(Box::new(TextValid))
                } else {
                    Some(Box::new(TextInvalid))
                }
            })
            .with_text("some text".into());

        let layout = SplitLayout::new(SplitDirection::Vertical)
            .with(
                SplitRule::Fixed(1),
                Box::new(LeafLayout::new(
                    Box::new(|w: &TwoButtonEdit| &w.edit_box),
                    Box::new(|w: &mut TwoButtonEdit| &mut w.edit_box),
                )),
            )
            .with(
                SplitRule::Fixed(1),
                Box::new(LeafLayout::new(
                    Box::new(|w: &TwoButtonEdit| &w.ok_button),
                    Box::new(|w: &mut TwoButtonEdit| &mut w.ok_button),
                )),
            )
            .with(
                SplitRule::Fixed(1),
                Box::new(LeafLayout::new(
                    Box::new(|w: &TwoButtonEdit| &w.cancel_button),
                    Box::new(|w: &mut TwoButtonEdit| &mut w.cancel_button),
                )),
            );

        TwoButtonEdit {
            id: get_new_widget_id(),
            layout: Box::new(layout),
            cached_sizes : None,
            ok_button,
            cancel_button,
            edit_box,
            focus_group: FocusGroupImpl::dummy(),
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

        None
    }
}

impl Widget for TwoButtonEdit {
    fn id(&self) -> usize {
        self.id
    }

    fn typename(&self) -> &'static str {
        "TwoButtonEdit"
    }

    fn min_size(&self) -> XY {
        XY::new(32, 10)
    }

    fn layout(&self, max_size: XY) -> XY {
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        let focus_update_op = default_key_to_focus_update(input_event);
        if focus_update_op.is_some() {
            return Some(Box::new(FocusUpdateMsg(focus_update_op.unwrap())));
        }

        //if we got here, it was NOT an focus update.

        match input_event {
            InputEvent::KeyInput(Key::Esc) => Some(Box::new(Cancel)),
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<TwoButtonEditMsg>();
        if our_msg.is_none() {
            warn!("expecetd TBEMsg, got {:?}", msg);
            return None;
        }

        match our_msg.unwrap() {
            TwoButtonEditMsg::FocusUpdateMsg(focus_update) => {
                self.focus_group.update_focus(*focus_update);
                None
            }

            TwoButtonEditMsg::TextValid => {
                self.ok_button.set_enabled(true);
                None
            }
            TwoButtonEditMsg::TextInvalid => {
                self.ok_button.set_enabled(false);
                None
            }
            _ => None,
        }
    }

    fn get_focused(&self) -> &dyn Widget {
        let focused_id = self.focus_group.get_focused();

        let focused_view: Option<&dyn Widget> = self.todo_wid_to_widget(focused_id);

        if focused_view.is_none() {
            warn!("failed getting focused_view in two_button_edit");
            return &self.cancel_button;
        };

        focused_view.unwrap()
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        let focused_id = self.focus_group.get_focused();

        let focused_view: Option<&mut dyn Widget> = if focused_id == self.ok_button.id() {
            Some(&mut self.ok_button)
        } else if focused_id == self.cancel_button.id() {
            Some(&mut self.cancel_button)
        } else if focused_id == self.edit_box.id() {
            Some(&mut self.edit_box)
        } else {
            None
        };

        //TODO this will panic if some id is wrong

        focused_view.unwrap()
    }

    fn render(&self, focused: bool, output: &mut dyn Output) {
        let focused_op = if focused {
            Some(self.focus_group.get_focused())
        } else {
            None
        };

        match &self.cached_sizes {
            None => warn!("failed rendering two_button_edit without cached_sizes"),
            Some(cached_sizes) => {
               for wir in &cached_sizes.widget_sizes {
                   match self.todo_wid_to_widget(wir.wid) {
                       None => warn!("failed to match WID {} to sub-widget in two_button_edit {}", wir.wid, self.id()),
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
