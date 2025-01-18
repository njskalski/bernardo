use crate::primitives::tree::filter_policy::FilterPolicy;
use crate::primitives::tree::tree_node::{FilterRef, TreeItFilter, TreeNode};
use log::{debug, error, warn};
use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;
use streaming_iterator::IntoStreamingIterator;
// TODO add a cut-off iterator size, so if a cyclical graph is provided this won't explode
// TODO add max depth parameter?

// All tree iterators must (as invariant) return Root node, even if it doesn't match filters.

pub struct LazyTreeIterator<'a, Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key> + 'static> {
    // child iter, whether added, item
    stack: Vec<(usize, bool, Item)>,
    fifo: VecDeque<(u16, Item)>,
    filter_op: Option<(FilterRef<Item>, FilterPolicy)>,
    expanded_op: Option<&'a HashSet<Key>>,
    limit: Option<usize>,
}

impl<'a, Key: Hash + Eq + Debug + Clone, Item: TreeNode<Key> + 'static> LazyTreeIterator<'a, Key, Item> {
    pub fn new(root: Item, filter_op: Option<(FilterRef<Item>, FilterPolicy)>) -> Self {
        Self {
            stack: vec![(0, true, root.clone())],
            fifo: VecDeque::from(vec![(0, root)]),
            filter_op,
            expanded_op: None,
            limit: None,
        }
    }

    pub fn with_expanded(self, expanded: &'a HashSet<Key>) -> Self {
        Self {
            expanded_op: Some(expanded),
            ..self
        }
    }

    fn matches(&self, node: &Item) -> bool {
        if let Some(filter) = self.filter_op.as_ref() {
            filter.0.call(node)
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
                debug!(target: "lazy_tree_iter", "picking up {:?}, added {}, child_it : {}, total children {}", node, added, child_it, if node.is_leaf() { 0 } else { node.child_iter().count() });

                if node.is_leaf() {
                    if self.matches(&node) {
                        debug!(target: "lazy_tree_iter", "+ node {:?} is a matching leaf, adding to queue",node);
                        self.fifo.push_back((self.stack.len() as u16, node));
                        break;
                    } else {
                        debug!(target: "lazy_tree_iter", "- node {:?} ia a non matching leaf, discarding",node);
                        continue;
                    }
                } else {
                    if self
                        .filter_op
                        .as_ref()
                        .map(|pair| pair.1 == FilterPolicy::MatchNode)
                        .unwrap_or(true)
                    {
                        if !self.matches(&node) {
                            debug!(target: "lazy_tree_iter", "- node {:?} ia a non matching inner node, in MatchNode thing, discarding",node);
                            continue;
                        } // All tree iterators must (as invariant) return Root node, even if it doesn't match filters.

                        if !added {
                            debug!(target: "lazy_tree_iter", "+ node {:?} ia a matching inner node, in MatchNode thing, adding",node);
                            self.fifo.push_back((self.stack.len() as u16, node.clone()));
                            added = true;
                        } else {
                            debug!(target: "lazy_tree_iter", "o node {:?} ia a matching inner node, in MatchNode thing, but was already added.",node);
                        }
                    } else {
                        debug_assert!(self
                            .filter_op
                            .as_ref()
                            .map(|pair| pair.1 == FilterPolicy::MatchNodeOrAncestors)
                            .unwrap_or(true));

                        if self.matches(&node) && child_it == 0 && !added {
                            debug!(target: "lazy_tree_iter", "+ node {:?} ia a matching inner node, in MatchNodeOrAncestors thing, adding.",node);
                            self.fifo.push_back((self.stack.len() as u16, node.clone()));
                            added = true;
                        }
                    }

                    debug_assert!(!node.is_leaf());
                    let is_expanded = self.expanded_op.map(|hs| hs.contains(node.id())).unwrap_or(true);
                    debug!(target: "lazy_tree_iter", "e node {:?} ia a matching inner node, expanded = {}",node, is_expanded);

                    if is_expanded {
                        if let Some(next_child) = node.child_iter().skip(child_it).next() {
                            debug!(target: "lazy_tree_iter", "> node {:?} has child id {}, pushing back to stack",node, child_it);
                            self.stack.push((child_it + 1, added, node.clone()));

                            if self
                                .filter_op
                                .as_ref()
                                .map(|pair| pair.1 == FilterPolicy::MatchNodeOrAncestors)
                                .unwrap_or(false)
                            {
                                if self.matches(&next_child) {
                                    debug!(target: "lazy_tree_iter", "o node's {:?} child {:?} matches in MatchNodeOrAncestors mode, will mark everything on stack as matching", node, child_it);
                                    for (depth, it) in self.stack.iter_mut().enumerate() {
                                        if it.1 == false {
                                            debug!(target: "lazy_tree_iter", "o marking {:?} as matching and pushing to results", it.2);
                                            it.1 = true;
                                            self.fifo.push_back((depth as u16, it.2.clone()));
                                        }
                                    }
                                }
                            }

                            debug!(target: "lazy_tree_iter", "> node {:?} pushing child {:?}", node, next_child);
                            self.stack.push((0, false, next_child));
                        } else {
                            debug!(target: "lazy_tree_iter", "o node {:?} had {} children, but no more.",node, child_it);
                            continue;
                        }
                    }
                }
            } else {
                break;
            }
        }

        if self.fifo.is_empty() {
            debug_assert!(self.stack.is_empty());
        }

        self.fifo.pop_front()
    }
}
