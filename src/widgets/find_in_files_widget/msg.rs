use crate::experiments::focus_group::FocusUpdate;
use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum Msg {
    Hit,
    Cancel,

    FocusUpdate(FocusUpdate),
}

impl AnyMsg for Msg {}
