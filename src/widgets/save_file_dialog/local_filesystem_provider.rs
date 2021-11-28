use std::borrow::Borrow;
use std::path::PathBuf;
use std::rc::Rc;

use crate::widgets::save_file_dialog::filesystem_provider::FilesystemProvider;
use crate::widgets::save_file_dialog::filesystem_tree::FilesystemNode;

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

impl FilesystemProvider for LocalFilesystemProvider {
    fn get_root(&self) -> Box<dyn Borrow<FilesystemNode>> {
        let x = FilesystemNode::new(self.root.clone());
        Box::new(Rc::new(x))
    }
}
