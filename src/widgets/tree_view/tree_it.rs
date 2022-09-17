/*
This iterator implements depth-first-order using a double ended queue to emulate recursion,
so I don't have to fight borrow-checker, that seem hard to marry with lazy instantiation.

It supports filtering and recursive filtering: if filter is present, then node will be visible in
    either case:
        - it passes filter test
        - one of it's descendants up to "filter_depth_op" deep (None = infinity)
TODO: filtering is NOT optimised, nodes will be queried on repeated occasions.

I got this idea in Zurich Operahouse, watching some ballet. Creativity sprouts from boredom.
 */

// TODO add tests

use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

use log::debug;
use streaming_iterator::StreamingIterator;

use crate::widgets::tree_view::tree_view_node::{MaybeBool, TreeItFilter, TreeViewNode};

// tu nie trzeba pogrzebacza, tu trzeba pogrzebu.

pub struct TreeIt<'a, Key: Hash + Eq + Debug, Item: TreeViewNode<Key>> {
    queue: Vec<(u16, Item)>,
    expanded: &'a HashSet<Key>,
    filter_op: Option<&'a TreeItFilter<Item>>,
    filter_depth_op: Option<usize>,
    first_advance: bool,
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
            first_advance: false,
        }
    }
}

impl<'a, Key: Hash + Eq + Debug + Clone, TItem: TreeViewNode<Key>> StreamingIterator for TreeIt<'a, Key, TItem> {
    type Item = (u16, TItem);

    fn advance(&mut self) {
        if !self.first_advance {
            self.first_advance = true;
            return;
        }

        if let Some((depth, node_ref)) = self.queue.pop() {
            if self.expanded.contains(node_ref.id()) {
                let mut children: Vec<TItem> = Vec::new();
                let mut child_it = node_ref.child_iter();
                while let Some(child) = child_it.next() {
                    children.push(child.clone());
                }

                for item in children.into_iter().rev() {
                    match self.filter_op {
                        // TODO this can be optimised by caching results of checks using item.key
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
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        self.queue.last()
    }
}
