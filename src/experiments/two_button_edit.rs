/*
this is experimental widget, which has two buttons (OK and Cancel) and TextBox
and OK is enabled ONLY if text TextBox content length > 4.

just an experiment to see if the design works.
 */

use crate::widget::button::ButtonWidget;
use crate::widget::edit_box::EditBoxWidget;
use crate::widget::widget::{Widget, MsgConstraints, get_new_widget_id, BaseWidget};
use crate::experiments::two_button_edit::TBEMsg::{TextValid, TextInvalid, Cancel};
use crate::io::input_event::InputEvent;
use crate::io::keys::Key;
use std::fs::read;
use crate::layout::split_layout::{SplitLayout, SplitDirection, SplitRule};
use crate::layout::leaf_layout::LeafLayout;
use crate::primitives::xy::XY;
use crate::io::output::Output;
use crate::layout::layout::Layout;
use crate::io::sub_output::SubOutput;
use crate::experiments::focus_group::{FocusGroupImpl, FocusGroup, FocusUpdate};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TBEMsg {
    OK,
    Cancel,
    TextValid,
    TextInvalid,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum NoMsg {}

impl MsgConstraints for TBEMsg {}

impl MsgConstraints for NoMsg {}

pub struct TwoButtonEdit  {
    id : usize,
    ok_button : ButtonWidget<TBEMsg>,
    cancel_button : ButtonWidget<TBEMsg>,
    edit_box : EditBoxWidget<TBEMsg>,
    layout : SplitLayout,
    focus_group : FocusGroupImpl,
}

impl TwoButtonEdit {
    pub fn new() -> Self {
        let ok_button = ButtonWidget::<TBEMsg>::new("OK".into())
            .with_on_hit(|_| Some(TBEMsg::OK))
            .with_enabled(false);

        let cancel_button = ButtonWidget::<TBEMsg>::new("Cancel".into()).with_on_hit(|_| Some(TBEMsg::Cancel));

        let edit_box = EditBoxWidget::<TBEMsg>::new().with_on_change(|eb| {
           if eb.get_text().len() > 4 {
               Some(TextValid)
           } else {
               Some(TextInvalid)
           }
        }).with_text("some text".into());

        let button_horizontal_split =
            SplitLayout::new(
                vec![Box::new(LeafLayout::from_widget(&ok_button)), Box::new(LeafLayout::from_widget(&cancel_button))],
                SplitDirection::Horizontal,
                vec![SplitRule::Proportional(1.0), SplitRule::Proportional(1.0)]
            ).unwrap();

        let vertical_split =
            SplitLayout::new(
                vec![Box::new(LeafLayout::from_widget(&edit_box)), Box::new(button_horizontal_split)],
                SplitDirection::Vertical,
                vec![SplitRule::Fixed(1), SplitRule::Fixed(1)]).unwrap();

        let mut focus_group = FocusGroupImpl::new(
            vec![
                ok_button.id(),
                cancel_button.id(),
                edit_box.id(),
            ]
        );

        focus_group.override_edges(
            edit_box.id(),
            vec![
                (FocusUpdate::Down, ok_button.id()),
                (FocusUpdate::Next, ok_button.id()),
            ]
        );

        focus_group.override_edges(
            ok_button.id(),
            vec![
                (FocusUpdate::Up, edit_box.id()),
                (FocusUpdate::Next, cancel_button.id()),
                (FocusUpdate::Prev, edit_box.id()),
                (FocusUpdate::Right, cancel_button.id()),
            ]
        );

        focus_group.override_edges(
            cancel_button.id(),
            vec![
                (FocusUpdate::Up, edit_box.id()),
                (FocusUpdate::Prev, ok_button.id()),
                (FocusUpdate::Left, ok_button.id()),
            ]
        );

        TwoButtonEdit{
            id : get_new_widget_id(),
            layout : vertical_split,
            ok_button,
            cancel_button,
            edit_box,
            focus_group,
        }
    }
}

impl BaseWidget for TwoButtonEdit {
    fn id(&self) -> usize {
        self.id
    }

    fn min_size(&self) -> XY {
        XY::new(32,10)
    }

    fn size(&self, max_size: XY) -> XY {
        self.min_size()
    }

    fn on_input_any(&self, input_event : InputEvent) -> Option<Box<dyn MsgConstraints>> {
        self.on_input(input_event).map(|msg| Box::new(msg))
    }
}

impl Widget<NoMsg> for TwoButtonEdit {
    type LocalMsg = TBEMsg;

    fn update(&mut self, msg: TBEMsg) -> Option<NoMsg> {
        match msg {
            TBEMsg::TextValid => {
                self.ok_button.set_enabled(true);
                None
            }
            TBEMsg::TextInvalid => {
                self.ok_button.set_enabled(false);
                None
            }
            _ => None
        }
    }

    fn on_input(&self, input_event: InputEvent) -> Option<TBEMsg> {
        let focused_id = self.focus_group.get_focused();

        let focused_view: Option<&dyn BaseWidget> = if focused_id == self.ok_button.id() {
            Some(&self.ok_button)
        } else if focused_id == self.cancel_button.id() {
            Some(&self.cancel_button)
        } else if focused_id == self.edit_box.id() {
            Some(&self.edit_box)
        } else { None };

        // focused_view.map(|fw|)

        match input_event {
            InputEvent::KeyInput(Key::Esc) => Some(Cancel),
            _ => None
        }
    }

    fn render(&self, focused: bool, output: &mut Output) {
        let ok_button_rect = self.layout
            .get_rect(output.size(), self.ok_button.id())
            .unwrap();

        let cancel_button_rect = self.layout
            .get_rect(output.size(), self.cancel_button.id())
            .unwrap();

        let edit_rect = self.layout
            .get_rect(output.size(), self.edit_box.id())
            .unwrap();

        let ok_focused = focused && self.focus_group.get_focused() == self.ok_button.id();
        let cancel_focused = focused && self.focus_group.get_focused() == self.cancel_button.id();
        let edit_focused = focused && self.focus_group.get_focused() == self.edit_box.id();

        self.ok_button.render(ok_focused, &mut SubOutput::new(Box::new(output), ok_button_rect));
        self.cancel_button.render(cancel_focused, &mut SubOutput::new(Box::new(output), cancel_button_rect));
        self.edit_box.render(edit_focused,&mut SubOutput::new(Box::new(output), edit_rect));
    }
}