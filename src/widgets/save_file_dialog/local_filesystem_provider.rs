use std::path::PathBuf;
use std::rc::Rc;

use crate::widgets::save_file_dialog::filesystem_provider::FilesystemProvider;
use crate::widgets::save_file_dialog::filesystem_tree::FilesystemNode;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

pub struct LocalFilesystemProvider {
    root: PathBuf,
    root_node: Rc<FilesystemNode>
}

impl LocalFilesystemProvider {
    pub fn new(root: PathBuf) -> Self {
        let root_node = Rc::new(FilesystemNode::new(root.clone()));

        LocalFilesystemProvider {
            root,
            root_node,
        }
    }
}

impl FilesystemProvider for LocalFilesystemProvider {
    fn get_root(&self) -> Box<dyn TreeViewNode<PathBuf>> {
        Box::new(self.root_node.clone())
    }
}
