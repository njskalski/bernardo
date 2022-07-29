/*
This is a piece of specialized code for TreeView of Rc<PathBuf>
 */

use std::path::PathBuf;
use std::sync::Arc;

use log::{debug, error, warn};
use streaming_iterator::StreamingIterator;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::path::SPath;
use crate::spath;
use crate::widgets::spath_tree_view_node::FileTreeNode;

use crate::widgets::tree_view::tree_view::TreeViewWidget;

impl TreeViewWidget<SPath, FileTreeNode> {
    pub fn expand_path(&mut self, path: &SPath) -> bool {
        debug!("setting path to {}", path);

        let root_node = self.get_root_node();

        if !root_node.spath().is_parent_of(path) {
            warn!("attempted to set path {}, but root is {}, ignoring.", path, root_node.spath());
            return false;
        }

        let exp_mut = self.expanded_mut();

        let mut parent_ref_iter = path.ancestors_and_self_ref();
        while let Some(anc) = parent_ref_iter.next() {
            if anc.is_file() {
                continue;
            }

            exp_mut.insert(anc.clone());
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::mock_fs::MockFS;
    use crate::fs::path::SPath;
    use crate::{FilesystemFront, spath};
    use crate::widgets::spath_tree_view_node::FileTreeNode;
    use crate::widgets::tree_view::tree_view::TreeViewWidget;

    #[test]
    fn test_set_path() {
        let mockfs = MockFS::new("/tmp")
            .with_file("folder1/folder2/file1.txt", "some text")
            .with_file("folder1/folder3/moulder.txt", "truth is out there")
            .to_fsf();

        let mut widget = TreeViewWidget::<SPath, FileTreeNode>::new(
            FileTreeNode::new(spath!(mockfs, "folder1").unwrap()));

        assert_eq!(widget.is_expanded(&spath!(mockfs, "folder1","folder2", "file1.txt").unwrap()), false);

        assert_eq!(widget.expand_path(&spath!(mockfs, "folder1", "folder2", "file1.txt").unwrap()), true);
        assert_eq!(widget.is_expanded(&spath!(mockfs, "folder1").unwrap()), true);
        assert_eq!(widget.is_expanded(&spath!(mockfs, "folder1","folder2").unwrap()), true);
        assert_eq!(widget.is_expanded(&spath!(mockfs, "folder1", "folder3").unwrap()), false);
    }
}