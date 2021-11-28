use std::borrow::Borrow;
use std::path::PathBuf;

use crate::widgets::save_file_dialog::filesystem_tree::FilesystemNode;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

pub trait FilesystemProvider {
    fn get_root(&self) -> Box<dyn TreeViewNode<PathBuf>>;
}

