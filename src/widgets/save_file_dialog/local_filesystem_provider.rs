use std::path::PathBuf;

use crate::widgets::save_file_dialog::filesystem_provider::FilesystemProvider;

pub struct LocalFilesystemProvider {
    root: PathBuf,
}

impl LocalFilesystemProvider {
    pub fn new(root: PathBuf) -> Self {
        LocalFilesystemProvider {
            root
        }
    }
}

impl FilesystemProvider for LocalFilesystemProvider {}
