use std::fmt::Debug;

use crate::AnyMsg;
use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::primitives::cursor_set::Cursor;

#[derive(Clone, Debug)]
pub enum EditorWidgetMsg {
    EditMsg(CommonEditMsg),

    ToCursorDropMode,
    ToEditMode,

    DropCursorFlip { cursor: Cursor },
    // not sure if this can't be simplified, why separate message?
    DropCursorMove { cem: CommonEditMsg },

    OpenContextMenu,
    ContextMenuClose,

    CompletionWidgetClose,
}


impl AnyMsg for EditorWidgetMsg {}