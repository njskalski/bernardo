use std::collections::HashSet;

use crate::widget::stupid_tree::get_stupid_tree;
use crate::widgets::tree_view::tree_it::TreeIt;

#[test]
fn tree_it_test_1() {
    let root = get_stupid_tree();

    let mut expanded: HashSet<usize> = HashSet::new();
    expanded.insert(0);
    expanded.insert(1);

    let try_out = |expanded_ref: &HashSet<usize>| {
        let items: Vec<(u16, String)> = TreeIt::new(&root, expanded_ref)
            .map(|(d, f)| (d, format!("{:?}", f.id())))
            .collect();
        let max_len = items.iter().fold(
            0,
            |acc, (_, item)| if acc > item.len() { acc } else { item.len() },
        );
        (items, max_len)
    };

    {
        let (items, max_len) = try_out(&expanded);
        assert_eq!(items.len(), 5);
        assert_eq!(max_len, 5);
    }

    expanded.insert(2);

    {
        let (items, max_len) = try_out(&expanded);
        assert_eq!(items.len(), 8);
        assert_eq!(max_len, 5);
    }

    expanded.insert(20003);

    {
        let (items, max_len) = try_out(&expanded);
        assert_eq!(items.len(), 9);
        assert_eq!(max_len, 7);
    }
}
