use std::fmt::Debug;

use crate::AnyMsg;
use crate::primitives::cursor_set::Cursor;
use crate::primitives::common_edit_msgs::CommonEditMsg;

#[derive(Clone, Debug)]
pub enum EditorWidgetMsg {
    EditMsg(CommonEditMsg),

    ToCursorDropMode,
    ToEditMode,

    DropCursorFlip { cursor: Cursor },
    // not sure if this can't be simplified, why separate message?
    DropCursorMove { cem: CommonEditMsg },
}


impl AnyMsg for EditorWidgetMsg {}