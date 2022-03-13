use std::fmt::Debug;
use std::hash::Hash;

use log::error;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MaybeBool {
    False,
    True,
    Maybe,
}

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
    fn matching_self_or_children(&self, filter: &TreeItFilter<Self>, max_depth: Option<usize>) -> MaybeBool {
        if filter(&self) {
            return MaybeBool::True;
        }

        if self.is_leaf() {
            return MaybeBool::False;
        }

        if max_depth == Some(0) {
            return MaybeBool::Maybe;
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

            match i.matching_self_or_children(filter, max_depth.map(|i| if i > 0 { i - 1 } else { 0 })) {
                MaybeBool::True => return MaybeBool::True,
                MaybeBool::Maybe => { any_chance = true; }
                _ => {}
            }
        }

        if any_chance {
            MaybeBool::Maybe
        } else {
            MaybeBool::False
        }
    }
}

pub type TreeItFilter<Node> = fn(&Node) -> bool;

// pub type TreeItFilter<Key: Hash, Node: TreeViewNode<Key>> = fn(&Node) -> bool;
// pub trait TreeItFilter<Key: Hash + Eq + Debug, Node: TreeViewNode<Key>>: Fn(&Node) -> bool {}
