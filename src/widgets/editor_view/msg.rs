use std::fmt::Debug;
use std::path::PathBuf;

use crate::AnyMsg;
use crate::fs::file_front::FileFront;
use crate::primitives::cursor_set::Cursor;
use crate::widgets::common_edit_msgs::CommonEditMsg;

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
}


impl AnyMsg for EditorViewMsg {}