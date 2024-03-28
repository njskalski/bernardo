use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

use crate::primitives::maybe_bool::MaybeBool;

// Keep it lightweight. It is expected to be implemented by Rc<some type>
pub trait TreeNode<Key: Hash + Eq + Debug>: Clone + Debug {
    fn id(&self) -> &Key;
    fn label(&self) -> Cow<str>;
    fn is_leaf(&self) -> bool;

    fn child_iter(&self) -> Box<dyn Iterator<Item = Self>>;

    fn is_complete(&self) -> bool;

    /*
    the answer is true, false, or "we don't know yet"
     */
    fn matching_self_or_children(&self, filter: &TreeItFilter<Self>, max_depth: Option<usize>) -> MaybeBool {
        if filter(self) {
            return MaybeBool::True;
        }

        if self.is_leaf() {
            return MaybeBool::False;
        }

        if max_depth == Some(0) {
            return MaybeBool::Maybe;
        }

        let mut any_chance = self.is_complete();
        for i in self.child_iter() {
            match i.matching_self_or_children(filter, max_depth.map(|i| if i > 0 { i - 1 } else { 0 })) {
                MaybeBool::True => return MaybeBool::True,
                MaybeBool::Maybe => {
                    any_chance = true;
                    // do not add break, we're still hunting for true
                }
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

pub type TreeItFilter<Node> = Box<dyn Fn(&Node) -> bool>;

// pub type TreeItFilter<Key: Hash, Node: TreeViewNode<Key>> = fn(&Node) -> bool;
// pub trait TreeItFilter<Key: Hash + Eq + Debug, Node: TreeViewNode<Key>>: Fn(&Node) -> bool {}
