use std::fmt::{Debug, Formatter};
use crate::primitives::arrow::Arrow;
use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum Msg {
    Hit,
    Arrow(Arrow),
    Backspace
}


impl AnyMsg for Msg {
}