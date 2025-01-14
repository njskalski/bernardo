use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::mock_fs::MockFS;
use crate::fs::path::SPath;
use std::collections::HashSet;

use crate::primitives::tree::filter_policy::FilterPolicy;
use crate::primitives::tree::lazy_tree_it::LazyTreeIterator;
use crate::primitives::tree::tree_it::eager_iterator;
use crate::primitives::tree::tree_node::TreeNode;
use crate::spath;
use crate::widgets::spath_tree_view_node::DirTreeNode;

#[test]
fn fsf_equal_iter_test() {
    let mock_fs = MockFS::generate_from_real("./test_envs/save_file_dialog_test_1").unwrap().to_fsf();
    let root = spath!(mock_fs).unwrap();

    let mut expanded: HashSet<SPath> = HashSet::new();

    fn run_test(root: &SPath, expanded: &HashSet<SPath>) {
        let eager: Vec<String> = eager_iterator(&DirTreeNode::new(root.clone()), Some(expanded), None, FilterPolicy::MatchNode)
            .map(|item| item.1.label().to_string())
            .collect();
        let lazy: Vec<String> = LazyTreeIterator::new(DirTreeNode::new(root.clone()))
            .with_expanded(expanded)
            .with_filter_policy(FilterPolicy::MatchNode)
            .map(|item| item.1.label().to_string())
            .collect();

        assert_eq!(eager, lazy);
    };

    run_test(&root, &expanded);

    expanded.insert(root.clone());

    run_test(&root, &expanded);
}
