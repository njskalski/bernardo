/*
This iterator implements depth-first-order using a double ended queue to emulate recursion,
so I don't have to fight borrow-checker, that seem hard to marry with lazy instantiation.

I got this idea in Zurich Operahouse, watching some ballet. Creativity sprouts from boredom.

Also, now it supports filtering and recursive filtering: if filter is present, then node
    will be visible in either case:
        - it passes filter test
        - one of it's descendants up to "filter_depth_op" deep (None = infinity)
 */

use crate::primitives::maybe_bool::MaybeBool;
use crate::primitives::tree::tree_node::{TreeItFilter, TreeNode};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

type QueueType<Item> = Item;

// tu nie trzeba pogrzebacza, tu trzeba pogrzebu.

pub struct TreeIt<'a, Key: Hash + Eq + Debug, Item: TreeNode<Key>> {
    queue: Vec<(u16, QueueType<Item>)>,
    expanded: &'a HashSet<Key>,
    filter_op: Option<&'a TreeItFilter<Item>>,
    filter_depth_op: Option<usize>,
}

impl<'a, Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> TreeIt<'a, Key, Item> {
    pub fn new(
        root: &Item,
        expanded: &'a HashSet<Key>,
        filter_op: Option<&'a TreeItFilter<Item>>,
        filter_depth_op: Option<usize>,
    ) -> TreeIt<'a, Key, Item> {
        TreeIt {
            queue: vec![(0, root.clone())],
            expanded,
            filter_op,
            filter_depth_op,
        }
    }
}

impl<'a, Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> Iterator for TreeIt<'a, Key, Item> {
    type Item = (u16, Item);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.queue.is_empty() {
            let head = self.queue.pop().unwrap();
            let (depth, node_ref) = head;

            // If it's expanded, I have to throw all children on the stack.
            if self.expanded.contains(node_ref.id()) {
                let idx_and_items: Vec<(usize, Item)> = node_ref.child_iter().enumerate().collect();
                for (_idx, item) in idx_and_items.into_iter().rev() {
                    if let Some(filter) = self.filter_op {
                        if item.matching_self_or_children(filter, self.filter_depth_op) == MaybeBool::False {
                            continue;
                        }
                    }

                    self.queue.push((depth + 1, item));
                }
            }

            return Some((depth, node_ref));
        }

        None
    }
}
