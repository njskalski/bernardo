use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::fsf_async_tree_iter::FsAsyncTreeIt;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::mock_fs::MockFS;
use crate::primitives::tree::filter_policy::FilterPolicy;
use crate::primitives::tree::tree_node::{ClosureFilter, TreeItFilter, TreeNode};
use crate::promise::streaming_promise::StreamingPromise;
use crate::widgets::file_tree_view::file_tree_view::FileTreeViewWidget;
use crate::widgets::spath_tree_view_node::FileTreeNode;
use std::sync::Arc;

fn mock_data() -> FsfRef {
    let mockfs = MockFS::new("/home")
        .with_file("folder1/file1_1.txt", "some text")
        .with_file("folder1/file1_2.txt", "some text2")
        .with_file("folder2/file2_1.txt", "some text2")
        .with_file("folder2/file2_2.txt", "some text2")
        .with_file(".hidden_folder3/file3_1.txt", "some text2");

    mockfs.to_fsf()
}

#[test]
fn promise_test_filter_match_node_1() {
    let fsf = mock_data();
    let filter = Arc::new(Box::new(ClosureFilter::new(|item: &FileTreeNode| {
        item.spath().label().starts_with('.') == false
    })) as Box<dyn TreeItFilter<FileTreeNode> + Send + Sync + 'static>);

    let mut promise = FsAsyncTreeIt::new(FileTreeNode::new(fsf.root()), Some((filter, FilterPolicy::MatchNode)), None);

    promise.drain(None);

    assert_eq!(
        promise.read().iter().map(|item| { item.1.label().to_string() }).collect::<Vec<_>>(),
        vec![
            "home".to_string(),
            "folder1".to_string(),
            "file1_1.txt".to_string(),
            "file1_2.txt".to_string(),
            "folder2".to_string(),
            "file2_1.txt".to_string(),
            "file2_2.txt".to_string(),
        ]
    );
}

#[test]
fn promise_test_filter_match_node_or_ancestors_1() {
    let fsf = mock_data();
    let filter = ClosureFilter::new(|item: &FileTreeNode| item.spath().label().starts_with('.') == false).arc_box();

    let mut promise = FsAsyncTreeIt::new(
        FileTreeNode::new(fsf.root()),
        Some((filter, FilterPolicy::MatchNodeOrAncestors)),
        None,
    );

    promise.drain(None);

    assert_eq!(
        promise.read().iter().map(|item| { item.1.label().to_string() }).collect::<Vec<_>>(),
        vec![
            "home".to_string(),
            ".hidden_folder3".to_string(),
            "file3_1.txt".to_string(),
            "folder1".to_string(),
            "file1_1.txt".to_string(),
            "file1_2.txt".to_string(),
            "folder2".to_string(),
            "file2_1.txt".to_string(),
            "file2_2.txt".to_string(),
        ]
    );
}

#[test]
fn promise_test_filter_2() {
    let m = MockFS::new("/tmp")
        .with_file("folder1/folder2/.gitignore", "file1.txt")
        .with_file("folder1/folder2/file1.txt", "not matched")
        .with_file("folder1/folder3/file1.txt", "matched")
        .to_fsf();

    let filter = FileTreeViewWidget::get_hidden_files_filter();

    let mut promise = FsAsyncTreeIt::new(
        FileTreeNode::new(m.root()),
        Some((filter, FilterPolicy::MatchNodeOrAncestors)),
        None,
    );

    promise.drain(None);

    let res: Vec<_> = promise.read().iter().map(|item| item.1.spath().to_string()).collect();
    let expectation: Vec<_> = vec![
        "",
        "folder1",
        "folder1/folder2",
        "folder1/folder2/file1.txt",
        "folder1/folder3",
        "folder1/folder3/file1.txt",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(res, expectation);
}
