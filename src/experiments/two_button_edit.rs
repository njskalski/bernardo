/*
this is experimental widget, which has two buttons (OK and Cancel) and TextBox
and OK is enabled ONLY if text TextBox content length > 4.

just an experiment to see if the design works.
 */

use crate::widget::button::ButtonWidget;
use crate::widget::edit_box::EditBoxWidget;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::experiments::two_button_edit::TwoButtonEditMsg::{TextValid, TextInvalid, Cancel, FocusUpdateMsg};
use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use std::fs::read;
// use crate::layout::split_layout::{SplitLayout, SplitDirection, SplitRule};
use crate::layout::leaf_layout::LeafLayout;
use crate::primitives::xy::XY;
use crate::io::output::Output;
use crate::layout::layout::Layout;
use crate::io::sub_output::SubOutput;
use crate::experiments::focus_group::{FocusGroupImpl, FocusGroup, FocusUpdate};
use crate::widget::any_msg::{AnyMsg, AsAny};
use std::borrow::Borrow;
use std::any::Any;
use log::warn;
use std::ops::Deref;
use crate::experiments::util::default_key_to_focus_update;
// use crate::layout::fixed_layout::{FixedLayout, FixedItem};
use crate::primitives::rect::Rect;
use crate::experiments::from_geometry::from_geometry;
use crate::layout::split_layout::{SplitLayout, SplitDirection};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TwoButtonEditMsg {
    OK,
    Cancel,
    TextValid,
    TextInvalid,
    FocusUpdateMsg(FocusUpdate)
}

impl AnyMsg for TwoButtonEditMsg {}

pub struct TwoButtonEdit {
    id: usize,
    ok_button: ButtonWidget,
    cancel_button: ButtonWidget,
    edit_box: EditBoxWidget,
    layout: Box<dyn Layout<TwoButtonEdit>>,
    focus_group: FocusGroupImpl,
}

impl TwoButtonEdit {
    pub fn new() -> Self {
        let ok_button = ButtonWidget::new("OK".into())
            .with_on_hit(|_| Some(Box::new(TwoButtonEditMsg::OK)))
            .with_enabled(false);

        let cancel_button = ButtonWidget::new("Cancel".into()).with_on_hit(|_| Some(Box::new(TwoButtonEditMsg::Cancel)));

        let edit_box = EditBoxWidget::new().with_on_change(|eb| {
            if eb.get_text().len() > 4 {
                Some(Box::new(TextValid))
            } else {
                Some(Box::new(TextInvalid))
            }
        }).with_text("some text".into());

        // let fixed_items : Vec<FixedItem<TwoButtonEdit>> = vec![
        //     FixedItem::new(
        //         Box::new(LeafLayout::new(
        //             Box::new(|tbe : &Self| {&tbe.edit_box}),
        //             Box::new(|tbe: &mut Self| {&mut tbe.edit_box})
        //         )),
        //         Rect::new(XY::new(1,1), XY::new(20, 1)),
        //     ),
        //     FixedItem::new(
        //         Box::new(LeafLayout::new(
        //             Box::new(|tbe: &Self| {&tbe.ok_button}),
        //             Box::new(|tbe: &mut Self| {&mut tbe.ok_button})
        //         )),
        //         Rect::new(XY::new(3,3), XY::new(8, 1)),
        //     ),
        //     FixedItem::new(
        //         Box::new(LeafLayout::new(
        //             Box::new(|tbe: &Self| {&tbe.cancel_button}),
        //             Box::new(|tbe: &mut Self| {&mut tbe.cancel_button})
        //         )),
        //         Rect::new(XY::new(14,3), XY::new(8, 1)),
        //     ),
        // ];

        // let layout = FixedLayout::new(size, fixed_items);

        let layout = SplitLayout::new(SplitDirection::Vertical);

        let size = XY::new(30, 8);

        let focus_group = from_geometry(&layout.get_all(size), Some(size));

        TwoButtonEdit {
            id: get_new_widget_id(),
            layout: Box::new(layout),
            ok_button,
            cancel_button,
            edit_box,
            focus_group,
        }
    }
}

impl Widget for TwoButtonEdit {
    fn id(&self) -> usize {
        self.id
    }

    fn min_size(&self) -> XY {
        XY::new(32, 10)
    }

    fn size(&self, max_size: XY) -> XY {
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        let focus_update_op = default_key_to_focus_update(input_event);
        if focus_update_op.is_some() {
            return Some(Box::new(FocusUpdateMsg(focus_update_op.unwrap())))
        }

        //if we got here, it was NOT an focus update.


        match input_event {
            InputEvent::KeyInput(Key::Esc) => Some(Box::new(Cancel)),
            _ => None
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
            _ => None
        }
    }

    fn get_focused(&self) -> &dyn Widget {
        let focused_id = self.focus_group.get_focused();

        let focused_view: Option<&dyn Widget> = if focused_id == self.ok_button.id() {
            Some(&self.ok_button)
        } else if focused_id == self.cancel_button.id() {
            Some(&self.cancel_button)
        } else if focused_id == self.edit_box.id() {
            Some(&self.edit_box)
        } else { None };

        if focused_view.is_none() {
            warn!("failed getting focused_view in two_button_edit");
            return &self.cancel_button
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
        } else { None };

        //TODO this will panic if some id is wrong

        focused_view.unwrap()
    }

    fn render(&self, focused: bool, frame_offset: XY, output: &mut Output) {
        let focused_op = if focused { None } else {
            Some(self.layout.get_focused(self).id())
        };

        self.layout.render(focused_op, frame_offset, output);

        // let frame = Rect::new(frame_offset, output.size());
        //
        // let ok_button_rect = self.layout
        //     .get_rect(output.size(), self.ok_button.id())
        //     .unwrap();
        //
        // let cancel_button_rect = self.layout
        //     .get_rect(output.size(), self.cancel_button.id())
        //     .unwrap();
        //
        // let edit_rect = self.layout
        //     .get_rect(output.size(), self.edit_box.id())
        //     .unwrap();
        //
        // let ok_focused = focused && self.focus_group.get_focused() == self.ok_button.id();
        // let cancel_focused = focused && self.focus_group.get_focused() == self.cancel_button.id();
        // let edit_focused = focused && self.focus_group.get_focused() == self.edit_box.id();
        //
        // if ok_button_rect.intersect(&frame).is_some() {
        //     self.ok_button.render(ok_focused, frame_offset, &mut SubOutput::new(Box::new(output), ok_button_rect));
        // }
        // if cancel_button_rect.intersect(&frame).is_some() {
        //     self.cancel_button.render(cancel_focused,  frame_offset, &mut SubOutput::new(Box::new(output), cancel_button_rect));
        // }
        // if edit_rect.intersect(&frame).is_some() {
        //     self.edit_box.render(edit_focused, frame_offset, &mut SubOutput::new(Box::new(output), edit_rect));
        // }
    }
}
