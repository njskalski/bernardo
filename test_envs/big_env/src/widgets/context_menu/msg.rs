use std::fmt::Debug;

use crate::widget::any_msg::AnyMsg;

#[derive(Debug, Clone)]
pub enum ContextMenuMsg {
    UpdateQuery(String),
    Close,
}

impl AnyMsg for ContextMenuMsg {}
