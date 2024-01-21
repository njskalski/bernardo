use std::fmt::Debug;

use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::widget::any_msg::AnyMsg;

#[derive(Clone, Copy, Debug)]
pub enum Navigation {
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
}

#[derive(Clone, Debug)]
pub enum FuzzySearchMsg {
    EditMsg(CommonEditMsg),
    EscalateContext,
    Navigation(Navigation),
    Hit,
    Close,
}

impl AnyMsg for FuzzySearchMsg {}
