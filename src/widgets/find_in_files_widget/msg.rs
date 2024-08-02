use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum Msg {
    Hit,
    Cancel,
}

impl AnyMsg for Msg {}