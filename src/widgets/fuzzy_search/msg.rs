use std::fmt::{Debug, Formatter};

use crate::AnyMsg;
use crate::widgets::common_edit_msgs::CommonEditMsg;

#[derive(Clone, Copy, Debug)]
pub enum FuzzySearchMsg {
    EditMsg(CommonEditMsg),
    EscalateContext,
}


impl AnyMsg for FuzzySearchMsg {}