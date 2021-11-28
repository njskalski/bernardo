use std::borrow::Borrow;
use std::path::PathBuf;

use crate::widget::tree_view_node::TreeViewNode;
use crate::widgets::save_file_dialog::filesystem_tree::FilesystemNode;

pub trait FilesystemProvider {
    fn get_root(&self) -> Box<dyn TreeViewNode<PathBuf>>;
}

