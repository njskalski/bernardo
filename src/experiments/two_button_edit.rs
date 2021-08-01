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
use crate::layout::split_layout::SplitLayout;
use crate::primitives::xy::XY;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TBEMsg {
    OK,
    Cancel,
    TextValid,
    TextInvalid,
}

impl MsgConstraints for TBEMsg {}

pub struct TwoButtonEdit  {
    id : usize,
    ok_button : ButtonWidget<TBEMsg>,
    cancel_button : ButtonWidget<TBEMsg>,
    edit_box : EditBoxWidget<TBEMsg>,

    vertical_split : SplitLayout,
    button_horizontal_split : SplitLayout,
}

impl TwoButtonEdit {
    fn new() -> Self {
        let ok_button = ButtonWidget::<TBEMsg>::new()
            .with_on_hit(|_| Some(TBEMsg::OK))
            .with_enabled(false);

        let cancel_button = ButtonWidget::<TBEMsg>::new().with_on_hit(|_| Some(TBEMsg::Cancel));

        let edit_box = EditBoxWidget::<TBEMsg>::new().with_on_change(|eb| {
           if eb.get_text().len() > 4 {
               Some(TextValid)
           } else {
               Some(TextInvalid)
           }
        });

        let button_horizontal_split = SplitLayout::new(vec![ok_button.id(), cancel_button.id()], XY::new(2, 1)).unwrap();

        TwoButtonEdit{
            id : get_new_widget_id(),
            button_horizontal_split,
            ok_button,
            cancel_button,
            edit_box,
        }
    }
}

impl BaseWidget for TwoButtonEdit {
    fn id(&self) -> usize {
        self.id
    }
}

impl<ParentMsg : MsgConstraints> Widget<ParentMsg> for TwoButtonEdit {
    type LocalMsg = TBEMsg;

    fn update(&mut self, msg: TBEMsg) -> Option<ParentMsg> {
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

    fn focusable(&self) -> bool {
        true
    }

    fn on_input(&self, input_event: InputEvent) -> Option<TBEMsg> {
        match input_event {
            InputEvent::KeyInput(Key::Esc) => Some(Cancel),
            _ => None
        }
    }
}