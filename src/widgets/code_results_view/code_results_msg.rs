use std::fmt::Debug;

use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum CodeResultsMsg {
    Hit { idx: usize },
}


impl AnyMsg for CodeResultsMsg {}
