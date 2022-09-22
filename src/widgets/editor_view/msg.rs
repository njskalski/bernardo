use crate::experiments::focus_group::FocusUpdate;
use crate::fs::path::SPath;
use crate::widget::any_msg::AnyMsg;

#[derive(Clone, Debug)]
pub enum EditorViewMsg {
    Save,
    SaveAs,
    OnSaveAsCancel,
    OnSaveAsHit { ff: SPath },

    FocusUpdateMsg(FocusUpdate),

    ToSimple,
    ToFind,
    ToFindReplace,

    /*
    semantics: starts from first cursor, finds the FIRST position of string phrase, and then stays on the first character position,
    no highlight
     */
    FindHit,
    ReplaceHit,
}


impl AnyMsg for EditorViewMsg {}