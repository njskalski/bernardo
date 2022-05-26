use crate::AnyMsg;
use crate::experiments::focus_group::FocusUpdate;
use crate::fs::file_front::FileFront;

#[derive(Clone, Debug)]
pub enum EditorViewMsg {
    Save,
    SaveAs,
    OnSaveAsCancel,
    OnSaveAsHit { ff: FileFront },

    FocusUpdateMsg(FocusUpdate),

    ToSimple,
    ToFind,
    ToFindReplace,
}


impl AnyMsg for EditorViewMsg {}