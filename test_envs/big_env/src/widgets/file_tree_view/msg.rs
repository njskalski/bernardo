use std::fmt::Debug;

use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum FileTreeViewMsg {
    ToggleHiddenFilesFilter,
}

impl AnyMsg for FileTreeViewMsg {}
