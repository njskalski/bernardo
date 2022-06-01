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

    /*
    semantics: starts from first cursor, finds the FIRST position of string phrase, and then stays on the first character position,
    no highlight
     */
    Find { phrase: String },
}


impl AnyMsg for EditorViewMsg {}