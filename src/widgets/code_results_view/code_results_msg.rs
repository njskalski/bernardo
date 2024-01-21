use std::fmt::Debug;

use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum CodeResultsMsg {
    Hit,
}

impl AnyMsg for CodeResultsMsg {}
