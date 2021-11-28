use std::path::PathBuf;
use std::rc::Rc;

use crate::widgets::tree_view::tree_view_node::TreeViewNode;

pub trait FilesystemProvider {
    fn get_root(&self) -> Rc<dyn TreeViewNode<PathBuf>>;
}

