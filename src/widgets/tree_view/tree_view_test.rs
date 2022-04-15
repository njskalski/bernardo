#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use crate::widget::stupid_tree::get_stupid_tree;
    use crate::widgets::tree_view::tree_it::TreeIt;
    use crate::widgets::tree_view::tree_view_node::TreeViewNode;

    #[test]
    fn tree_it_test_1() {
        let root = get_stupid_tree();

        let mut expanded: HashSet<usize> = HashSet::new();
        expanded.insert(0);
        expanded.insert(1);

        let try_out = |expanded_ref: &HashSet<usize>| {
            let items: Vec<(u16, String)> = TreeIt::new(&root, expanded_ref, None, None)
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
            assert_eq!(items.len(), 7);
            /*
        0: 0 -
        1:  1 -
        2:    10001
        3:    10002
        4:   2 *
        5:   3
        6:  4 *
        len 7.
         */

            assert_eq!(max_len, 5);
        }

        expanded.insert(2);

        {
            let (items, max_len) = try_out(&expanded);
            /*
        0: 0 -
        1:  1 -
        2:    10001
        3:    10002
        4:  2 -
        5:    20001
        6:    20002
        7:    20003 *
        8:  3
        9:  4 *
        len 10.
         */

            assert_eq!(items.len(), 10);
            assert_eq!(max_len, 5);
        }

        expanded.insert(20003);

        {
            /*
        0: 0 -
        1:  1 -
        2:    10001
        3:    10002
        4:  2 -
        5:    20001
        6:    20002
        7:    20003 -
        8:          2000301
        9:  3
       10:  4 *
        len 11.
         */

            let (items, max_len) = try_out(&expanded);
            assert_eq!(items.len(), 11);
            assert_eq!(max_len, 7);
        }
    }
}