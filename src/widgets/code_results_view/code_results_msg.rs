use std::fmt::Debug;

use crate::fs::path::SPath;
use crate::widget::any_msg::AnyMsg;
use crate::widgets::main_view::main_view::DocumentIdentifier;

#[derive(Debug)]
pub enum CodeResultsMsg {
    Hit,
}


impl AnyMsg for CodeResultsMsg {}
