use std::fmt::Debug;

use crate::primitives::arrow::VerticalArrow;
use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum Msg {
    Hit,
    Arrow(VerticalArrow),
    UnwrapOneLevel,
}

impl AnyMsg for Msg {}
