/*
This iterator implements depth-first-order using a double ended queue to emulate recursion,
so I don't have to fight borrow-checker, that seem hard to marry with lazy instantiation.

I got this idea in Zurich Operahouse, watching some ballet. Creativity sprouts from boredom.

Also, now it supports filtering and recursive filtering: if filter is present, then node
    will be visible in either case:
        - it passes filter test
        - one of it's descendants up to "filter_depth_op" deep (None = infinity)
 */
use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

use crate::widgets::tree_view::tree_view_node::{MaybeBool, TreeItFilter, TreeViewNode};

// tu nie trzeba pogrzebacza, tu trzeba pogrzebu.

pub struct TreeIt<'a, Key: Hash + Eq + Debug, Item: TreeViewNode<Key>> {
    queue: Vec<(u16, Item)>,
    expanded: &'a HashSet<Key>,
    filter_op: Option<&'a TreeItFilter<Item>>,
    filter_depth_op: Option<usize>,
}

impl<'a, Key: Hash + Eq + Debug + Clone, Item: TreeViewNode<Key>> TreeIt<'a, Key, Item> {
    pub fn new(
        root: &Item,
        expanded: &'a HashSet<Key>,
        filter_op: Option<&'a TreeItFilter<Item>>,
        filter_depth_op: Option<usize>,
    ) -> TreeIt<'a, Key, Item> {
        let mut queue: Vec<(u16, Item)> = Vec::new();

        queue.push((0, root.clone()));

        TreeIt {
            queue,
            expanded,
            filter_op,
            filter_depth_op,
        }
    }
}

impl<'a, Key: Hash + Eq + Debug + Clone, TItem: TreeViewNode<Key>> Iterator for TreeIt<'a, Key, TItem> {
    type Item = (u16, TItem);

    fn next(&mut self) -> Option<Self::Item> {
        while self.queue.is_empty() == false {
            let head = self.queue.pop().unwrap();
            let (depth, node_ref) = head;

            // If it's expanded, I have to throw all children on the stack.
            if self.expanded.contains(node_ref.id()) {
                let idx_and_items: Vec<(usize, TItem)> = node_ref.child_iter().enumerate().collect();
                for (_idx, item) in idx_and_items.into_iter().rev() {
                    match self.filter_op {
                        Some(filter) => {
                            if item.matching_self_or_children(filter.borrow(), self.filter_depth_op) == MaybeBool::False {
                                continue;
                            }
                        }
                        None => {}
                    }

                    self.queue.push(
                        (depth + 1, item)
                    );
                }
            }

            return Some((depth, node_ref.clone()));
        }

        None
    }
}
