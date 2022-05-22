use crate::AnyMsg;
use crate::fs::file_front::FileFront;

#[derive(Clone, Debug)]
pub enum EditorViewMsg {
    Save,
    SaveAs,
    OnSaveAsCancel,
    OnSaveAsHit { ff: FileFront },
}


impl AnyMsg for EditorViewMsg {}