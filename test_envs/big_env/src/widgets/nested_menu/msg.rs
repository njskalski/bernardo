use std::fmt::Debug;

use crate::primitives::arrow::VerticalArrow;
use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum Msg {
    Hit,
    Arrow(VerticalArrow),
    UnwrapOneLevel,
    QueryEdit(CommonEditMsg),
}

impl AnyMsg for Msg {}
