use crate::primitives::tree::filter_policy::FilterPolicy;
use crate::primitives::tree::tree_node::{FilterRef, TreeItFilter, TreeNode};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
// TODO this should be optimised but I am too tired to fix it.
// TODO add a cut-off iterator size, so if a cyclical graph is provided this won't explode
// TODO add max depth parameter?

// All tree iterators must (as invariant) return Root node, even if it doesn't match filters.

pub fn eager_iterator<'a, Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key> + 'static>(
    root: &Item,
    expanded_op: Option<&HashSet<Key>>,
    filter_op: Option<FilterRef<Item>>,
    filter_policy: FilterPolicy,
) -> impl Iterator<Item = (u16, Item)> {
    // bool in the middle stands for "filter hit";
    let mut result: Vec<(u16, bool, Item)> = Vec::with_capacity(128);

    fn matches<Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key> + 'static>(node: &Item, filter_op: Option<&FilterRef<Item>>) -> bool {
        if let Some(filter) = filter_op.as_ref() {
            filter.call(node)
        } else {
            true
        }
    }

    fn recursive<'a, Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key> + 'static>(
        item: &Item,
        is_root: bool,
        expanded_op: Option<&HashSet<Key>>,
        depth: u16,
        filter_op: Option<&FilterRef<Item>>,
        filter_policy: FilterPolicy,
        result: &mut Vec<(u16, bool, Item)>,
    ) {
        if item.is_leaf() {
            if matches(item, filter_op) {
                result.push((depth, true, item.clone()));
                return;
            } else {
                return;
            }
        } else {
            match filter_policy {
                FilterPolicy::MatchNode => {
                    if !matches(item, filter_op) {
                        return;
                    }
                }
                _ => {}
            }

            let filter_hit = is_root || filter_op.map(|filter| filter.call(item)).unwrap_or(true);

            result.push((depth, filter_hit, item.clone()));
            let self_added_at = result.len() - 1;

            if expanded_op.map(|set| set.contains(item.id())).unwrap_or(true) {
                for child in item.child_iter() {
                    recursive(&child, false, expanded_op, depth + 1, filter_op, filter_policy, result);

                    // if we're filtering, we add the current node only if it's on the path to a matching leaf AND was not added already.
                    if !filter_hit && filter_op.is_some() && filter_policy == FilterPolicy::MatchNodeOrAncestors {
                        let mut was_hit_afterwards = false;
                        if self_added_at + 1 < result.len() {
                            for idx in (self_added_at + 1)..result.len() {
                                if result[idx].1 {
                                    was_hit_afterwards = true;
                                    break;
                                }
                            }
                        }
                        if was_hit_afterwards {
                            result[self_added_at].1 = true;
                        } else {
                            // result.truncate(self_added_at);
                        }
                    }
                }
            }
        }
    }

    recursive(root, true, expanded_op, 0, filter_op.as_ref(), filter_policy, &mut result);

    result.into_iter().filter(|item| item.1).map(|item| (item.0, item.2))
}
