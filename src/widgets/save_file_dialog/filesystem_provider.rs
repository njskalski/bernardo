
use std::path::PathBuf;


use crate::widgets::tree_view::tree_view_node::TreeViewNode;

pub trait FilesystemProvider {
    fn get_root(&self) -> Box<dyn TreeViewNode<PathBuf>>;
}

