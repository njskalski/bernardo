use std::fmt::Debug;

use crate::widget::any_msg::AnyMsg;

#[derive(Debug, Clone)]
pub enum ContextMenuMsg {
    UpdateQuery(String),
}

impl AnyMsg for ContextMenuMsg {}
