use std::borrow::{Borrow, BorrowMut};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use log::{debug, warn};

use crate::widgets::save_file_dialog::filesystem_provider::FilesystemProvider;
use crate::widgets::save_file_dialog::filesystem_tree::FilesystemNode;
use crate::widgets::tree_view::tree_view_node::{ChildRc, TreeViewNode};

pub struct LocalFilesystemProvider {
    root: PathBuf,
    root_node: Rc<FilesystemNode>,
}

impl LocalFilesystemProvider {
    pub fn new(root: PathBuf) -> Self {
        let root_node = Rc::new(FilesystemNode::new(root.clone()));

        LocalFilesystemProvider {
            root,
            root_node,
        }
    }

    // substitutes current node corresponding to path with one where children cache is filled.
    // if path is invalid in current tree, it fails.
    pub fn expand_last(&mut self, path: &Path) -> bool {
        // TODO here I am assuming that self.root is prefix to path. This should be checked.

        if !path.starts_with(self.root.as_path()) {
            warn!("path {:?} is not prefixed by root {:?}", path, self.root)
        }

        let skip = self.root.components().count();

        let mut curr_node = self.root_node.clone() as Rc<dyn TreeViewNode<PathBuf>>;
        let mut curr_prefix = self.root.clone();

        let num_components = path.components().count();

        debug!("comp : {:?}", path.components());

        for (idx, c) in path.components().enumerate().skip(skip) {
            let last = idx == num_components - 1;
            curr_prefix.push(c);

            match curr_node.get_child_by_key(curr_prefix.borrow()) {
                None => {
                    warn!("{:?} has no child {:?}!", curr_node.id(), curr_prefix);
                    return false;
                }
                Some(new_node) => {
                    curr_node = new_node;
                }
            }
        }

        // if we got here, curr_node points to node corresponding to path.
        debug_assert!(curr_node.id() == path);
        curr_node.todo_update_cache();

        true
    }
}

impl FilesystemProvider for LocalFilesystemProvider {
    fn get_root(&self) -> Rc<dyn TreeViewNode<PathBuf>> {
        self.root_node.clone()
    }

    fn expand(&mut self, path: &Path) -> bool {
        self.expand_last(path)
    }
}
