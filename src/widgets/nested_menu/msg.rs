use std::fmt::{Debug, Formatter};
use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum Msg {
    Hit,
    ArrowUp,
    ArrowDown,
    Backspace
}


impl AnyMsg for Msg {
}