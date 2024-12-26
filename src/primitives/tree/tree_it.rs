use crate::primitives::tree::tree_it::FilterPolicy::MatchNode;
use crate::primitives::tree::tree_node::{TreeItFilter, TreeNode};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

// TODO this should be optimised but I am too tired to fix it.
// TODO add a cut-off iterator size, so if a cyclical graph is provided this won't explode
// TODO add max depth parameter?
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum FilterPolicy {
    MatchNode,
    MatchNodeOrAncestors,
}

pub fn eager_iterator<'a, Key: Hash + Eq + Debug, Item: TreeNode<Key>>(
    root: &Item,
    expanded_op: Option<&HashSet<Key>>,
    filter_op: Option<&TreeItFilter<Item>>,
    filter_policy: FilterPolicy,
) -> impl Iterator<Item = (u16, Item)> {
    fn recursive<'a, Key: Hash + Eq + Debug, Item: TreeNode<Key>>(
        item: &Item,
        expanded_op: Option<&HashSet<Key>>,
        depth: u16,
        filter_op: Option<&TreeItFilter<Item>>,
        filter_policy: FilterPolicy,
    ) -> Vec<(u16, Item)> {
        if item.is_leaf() {
            if filter_op.map(|filter| filter(item)).unwrap_or(true) {
                return vec![(depth, item.clone())];
            } else {
                return vec![];
            }
        } else {
            match filter_policy {
                FilterPolicy::MatchNode => {
                    if !filter_op.map(|filter| filter(item)).unwrap_or(true) {
                        return vec![];
                    }
                }
                _ => {}
            }

            let mut self_added_already = false;
            let mut result: Vec<(u16, Item)> = Default::default();

            if filter_op.map(|filter| filter(item)).unwrap_or(true) {
                result.push((depth, item.clone()));
                self_added_already = true;
            }

            if expanded_op.map(|set| set.contains(item.id())).unwrap_or(true) {
                for child in item.child_iter() {
                    let mut partial_result = recursive(&child, expanded_op, depth + 1, filter_op, filter_policy);

                    // if we're filtering, we add the current node only if it's on the path to a matching leaf AND was not added already.
                    if filter_op.is_some() && filter_policy == FilterPolicy::MatchNodeOrAncestors {
                        if !partial_result.is_empty() {
                            if self_added_already == false {
                                result.push((depth, item.clone()));
                                self_added_already = true;
                            }

                            result.append(&mut partial_result);
                        }
                    } else {
                        result.append(&mut partial_result);
                    }
                }
            }

            result
        }
    }

    recursive(root, expanded_op, 0, filter_op, filter_policy).into_iter()
}
