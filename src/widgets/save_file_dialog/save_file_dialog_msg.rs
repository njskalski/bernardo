use crate::AnyMsg;
use crate::experiments::focus_group::FocusUpdate;
use crate::fs::path::SPath;

#[derive(Clone, Debug)]
pub enum SaveFileDialogMsg {
    FocusUpdateMsg(FocusUpdate),
    // Sent when a left hand-side file-tree subtree is expanded (default: on Enter key)
    TreeExpanded(SPath),
    // Sent when a left hand-side file-tree subtree selection changed
    TreeHighlighted(SPath),
    FileListHit(SPath),
    EditBoxHit,

    Cancel,
    Save,

    ConfirmOverride,
    CancelOverride,
}

impl AnyMsg for SaveFileDialogMsg {}
