use std::fmt::{Debug, Formatter};

use crate::AnyMsg;
use crate::widgets::common_edit_msgs::CommonEditMsg;

#[derive(Clone, Copy, Debug)]
pub enum Navigation {
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
}

#[derive(Clone, Copy, Debug)]
pub enum FuzzySearchMsg {
    EditMsg(CommonEditMsg),
    EscalateContext,
    Navigation(Navigation),
    Hit,
    Close,
}


impl AnyMsg for FuzzySearchMsg {}