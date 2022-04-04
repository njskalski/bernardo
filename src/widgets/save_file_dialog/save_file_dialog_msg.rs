use std::rc::Rc;

use crate::AnyMsg;
use crate::experiments::focus_group::FocusUpdate;
use crate::fs::file_front::FileFront;

#[derive(Clone, Debug)]
pub enum SaveFileDialogMsg {
    FocusUpdateMsg(FocusUpdate),
    // Sent when a left hand-side file-tree subtree is expanded (default: on Enter key)
    TreeExpanded(FileFront),
    // Sent when a left hand-side file-tree subtree selection changed
    TreeHighlighted(FileFront),
    FileListHit(FileFront),
    EditBoxHit,

    Cancel,
    Save,

    ConfirmOverride,
    CancelOverride,
}

impl AnyMsg for SaveFileDialogMsg {}
