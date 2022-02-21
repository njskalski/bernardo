use std::path::{Path, PathBuf};

use crate::widget::widget::WID;

pub trait FilesystemProvider2 {
    fn root(&self) -> &Path;
    fn filesystem_view(&self) -> FilesystemView;

    // bool at the end indicates if the list is complete, or should we expect more files to follow.
    fn get_list(&self, path: &Path, caller_id: WID) -> (Box<dyn Iterator<Item=FilesystemListItem>>, bool);

    fn get_tree(&self, path: &Path, caller_id: WID) -> (FilesystemTreeNode2, bool);
}
