use crate::primitives::tree::filter_policy::FilterPolicy;
use crate::primitives::tree::tree_node::{TreeItFilter, TreeNode};

use crate::primitives::tree::filter_policy::FilterPolicy::MatchNodeOrAncestors;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;
// TODO add a cut-off iterator size, so if a cyclical graph is provided this won't explode
// TODO add max depth parameter?

pub struct LazyTreeIterator<'a, Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> {
    // child iter, whether added, item
    stack: Vec<(usize, bool, Item)>,
    fifo: VecDeque<(u16, Item)>,
    filter_policy: FilterPolicy,
    filter_op: Option<&'a TreeItFilter<Item>>,
    expanded_op: Option<&'a HashSet<Key>>,
}

impl<'a, Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> LazyTreeIterator<'a, Key, Item> {
    pub fn new(root: Item) -> Self {
        Self {
            stack: vec![(0, false, root)],
            fifo: VecDeque::new(),
            filter_policy: FilterPolicy::MatchNode,
            filter_op: None,
            expanded_op: None,
        }
    }

    pub fn with_filter(self, filter: &'a TreeItFilter<Item>) -> Self {
        Self {
            filter_op: Some(filter),
            ..self
        }
    }

    pub fn with_expanded(self, expanded: &'a HashSet<Key>) -> Self {
        Self {
            expanded_op: Some(expanded),
            ..self
        }
    }

    pub fn with_filter_policy(self, filter_policy: FilterPolicy) -> Self {
        Self { filter_policy, ..self }
    }

    fn matches(&self, node: &Item) -> bool {
        if let Some(filter) = self.filter_op.as_ref() {
            filter(node)
        } else {
            true
        }
    }
}

impl<'a, Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key>> Iterator for LazyTreeIterator<'a, Key, Item> {
    type Item = (u16, Item);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.fifo.is_empty() {
            return self.fifo.pop_front();
        };

        loop {
            if let Some((child_it, mut added, node)) = self.stack.pop() {
                if node.is_leaf() {
                    if self.matches(&node) {
                        self.fifo.push_back((self.stack.len() as u16, node));
                        break;
                    } else {
                        continue;
                    }
                } else {
                    if self.filter_policy == FilterPolicy::MatchNode {
                        if !self.matches(&node) {
                            continue;
                        }

                        if !added {
                            self.fifo.push_back((self.stack.len() as u16, node.clone()));
                            added = true;
                        }
                    } else {
                        debug_assert!(self.filter_policy == FilterPolicy::MatchNodeOrAncestors);

                        if self.matches(&node) && child_it == 0 && !added {
                            self.fifo.push_back((self.stack.len() as u16, node.clone()));
                            added = true;
                        }
                    }

                    let is_expanded = self.expanded_op.map(|hs| hs.contains(node.id())).unwrap_or(true);

                    if is_expanded {
                        if let Some(next_child) = node.child_iter().skip(child_it).next() {
                            self.stack.push((child_it + 1, added, node.clone()));

                            if self.filter_policy == FilterPolicy::MatchNodeOrAncestors {
                                if self.matches(&next_child) {
                                    for (depth, it) in self.stack.iter_mut().enumerate() {
                                        if it.1 == false {
                                            it.1 = true;
                                            self.fifo.push_back((depth as u16, it.2.clone()));
                                        }
                                    }
                                }
                            }

                            self.stack.push((0, false, next_child));
                        } else {
                            continue;
                        }
                    }
                }
            } else {
                break;
            }
        }

        self.fifo.pop_front()
    }
}
