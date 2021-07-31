use crate::io::input_event::InputEvent;
use std::fmt::Debug;

pub trait MsgConstraints : Copy + Clone + Debug {}

pub trait Widget<ParentMsg: MsgConstraints> {
    type LocalMsg;

    fn update(&mut self, msg: Self::LocalMsg) -> Option<ParentMsg>;

    fn focusable(&self) -> bool;

    /*
    returns Some() if event was consumed.
    separated from update, so the InputEvent can get escalated.
     */
    fn on_input(&self, input_event: InputEvent) -> Option<Self::LocalMsg>;
}
