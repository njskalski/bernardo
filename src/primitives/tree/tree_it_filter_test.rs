#[cfg(test)]
mod test {
    use crate::mocks::mock_tree_item::{get_mock_data_set_2, get_mock_data_set_3, MockTreeItem};
    use crate::primitives::common_query::CommonQuery;
    use crate::primitives::tree::tree_it::{eager_iterator, FilterPolicy};
    use crate::primitives::tree::tree_node::TreeItFilter;

    #[test]
    fn tree_it_filter_test_1() {
        let tree = get_mock_data_set_2();

        // name: "root1".to_string(),
        //         name: "option1".to_string(),
        //         name: "subtree1".to_string(),
        //             name: "subsubtree1".to_string(),
        //                     name: "subsubtree1child1".to_string(),
        //                     name: "subsubtree1child2".to_string(),
        //         name: "option2".to_string(),
        //         name: "subtree2".to_string(),
        //                 name: "subsubtree2".to_string(),
        //                     name: "subsubtree2child1".to_string(),
        //                 name: "subtree2child1".to_string(),
        //                 name: "subtree2child2".to_string(),

        {
            let query = CommonQuery::Fuzzy("oo".to_string()); // matches "child"
            let filter: TreeItFilter<MockTreeItem> = Box::new(move |item| query.matches(item.name.as_str()));

            let mut iterator = eager_iterator(&tree, None, Some(&filter), FilterPolicy::MatchNodeOrAncestors);

            let names: Vec<String> = iterator.map(|(depth, item)| item.name.clone()).collect();

            assert_eq!(names, vec!["root1".to_string(), "option1".to_string(), "option2".to_string()]);
        }
    }

    #[test]
    fn tree_it_filter_test_2() {
        let tree = get_mock_data_set_3();

        // name: "root1".to_string(),
        //         name: "option1".to_string(),
        //         name: "subtree1".to_string(),
        //             name: "subsubtree1".to_string(),
        //                     name: "subsubtree1child1".to_string(),
        //                     name: ".subsubtree1hiddenchild2".to_string(),
        //         name: ".hiddenoption2".to_string(),
        //         name: ".hiddensubtree2".to_string(),
        //                 name: "subsubtree2".to_string(),
        //                     name: "subsubtree2child1".to_string(),
        //                 name: "subtree2child1".to_string(),
        //                 name: "subtree2child2".to_string(),

        {
            let filter: TreeItFilter<MockTreeItem> = Box::new(move |item| item.name.starts_with('.') == false);

            let mut iterator = eager_iterator(&tree, None, Some(&filter), FilterPolicy::MatchNode);

            let names: Vec<String> = iterator.map(|(depth, item)| item.name.clone()).collect();

            assert_eq!(
                names,
                vec![
                    "root1".to_string(),
                    "option1".to_string(),
                    "subtree1".to_string(),
                    "subsubtree1".to_string(),
                    "subsubtree1child1".to_string()
                ]
            );
        }
    }
}
