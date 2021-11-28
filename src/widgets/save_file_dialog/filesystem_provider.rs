use std::borrow::Borrow;

use crate::widgets::save_file_dialog::filesystem_tree::FilesystemNode;

pub trait FilesystemProvider {
    fn get_root(&self) -> Box<dyn Borrow<FilesystemNode>>;
}

