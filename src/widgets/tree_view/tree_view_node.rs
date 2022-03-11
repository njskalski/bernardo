use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::rc::Rc;

use log::error;

// Keep it lightweight. It is expected to be implemented by Rc<some type>
pub trait TreeViewNode<Key: Hash + Eq + Debug>: Clone + Debug {
    fn id(&self) -> &Key;
    fn label(&self) -> String;
    fn is_leaf(&self) -> bool;

    fn num_child(&self) -> (bool, usize);
    fn get_child(&self, idx: usize) -> Option<Self>;

    fn is_complete(&self) -> bool;

    /*
    the answer is true, false, or "we don't know yet"
     */
    fn has_matching_children(&self, filter: &TreeItFilter<Key, Self>, max_depth: Option<usize>) -> Option<bool> {
        if filter(&self) {
            return Some(true);
        }

        if self.is_leaf() {
            return Some(false);
        }

        if max_depth == Some(0) {
            return None;
        }

        let mut any_chance = false;
        let (done, num_children) = self.num_child();
        any_chance = done;

        for idx in 0..num_children {
            let i = match self.get_child(idx) {
                Some(i) => i,
                None => {
                    error!("no element at expected index {}", idx);
                    continue;
                }
            };

            match i.has_matching_children(filter, max_depth.map(|i| if i > 0 { i - 1 } else { 0 })) {
                Some(true) => return Some(true),
                None => { any_chance = true; }
                _ => {}
            }
        }

        if any_chance {
            None
        } else {
            Some(false)
        }
    }
}

// type TreeItFilter<Key, Node: TreeViewNode<Key>> = fn(&Node) -> bool;
pub trait TreeItFilter<Key: Hash + Eq + Debug, Node: TreeViewNode<Key>>: Fn(&Node) -> bool {}
