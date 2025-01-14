use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::mock_fs::MockFS;
use crate::fs::path::SPath;
use crate::spath;
use crate::widgets::spath_tree_view_node::FileTreeNode;
use crate::widgets::tree_view::tree_view::TreeViewWidget;

#[test]
fn test_set_path() {
    let mockfs = MockFS::new("/tmp")
        .with_file("folder1/folder2/file1.txt", "some text")
        .with_file("folder1/folder3/moulder.txt", "truth is out there")
        .to_fsf();

    let mut widget = TreeViewWidget::<SPath, FileTreeNode>::new(FileTreeNode::new(spath!(mockfs, "folder1").unwrap()));

    assert_eq!(
        widget.is_expanded(&spath!(mockfs, "folder1", "folder2", "file1.txt").unwrap()),
        false
    );

    assert_eq!(
        widget.expand_path(&spath!(mockfs, "folder1", "folder2", "file1.txt").unwrap()),
        true
    );
    assert_eq!(widget.is_expanded(&spath!(mockfs, "folder1").unwrap()), true);
    assert_eq!(widget.is_expanded(&spath!(mockfs, "folder1", "folder2").unwrap()), true);
    assert_eq!(widget.is_expanded(&spath!(mockfs, "folder1", "folder3").unwrap()), false);
}
