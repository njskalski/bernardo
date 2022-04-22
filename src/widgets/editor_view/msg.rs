use std::fmt::Debug;

use crate::AnyMsg;
use crate::fs::file_front::FileFront;
use crate::primitives::cursor_set::Cursor;
use crate::primitives::common_edit_msgs::CommonEditMsg;

#[derive(Clone, Debug)]
pub enum EditorViewMsg {
    EditMsg(CommonEditMsg),
    Save,
    SaveAs,
    OnSaveAsCancel,
    OnSaveAsHit { ff: FileFront },

    ToCursorDropMode,
    ToEditMode,

    DropCursor { cursor: Cursor },
    DropCursorMove { cem: CommonEditMsg },
}


impl AnyMsg for EditorViewMsg {}