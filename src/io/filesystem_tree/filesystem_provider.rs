use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::io::filesystem_tree::filesystem_list_item::FilesystemListItem;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

pub trait FilesystemProvider {
    fn get_root(&self) -> Rc<dyn TreeViewNode<PathBuf>>;

    fn expand(&mut self, path: &Path) -> bool;

    fn get_files(&self, path: &Path) -> Box<dyn Iterator<Item=FilesystemListItem>>;
}

