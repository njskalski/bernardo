use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::rc::Rc;

pub type ChildRc<Key> = Rc<dyn TreeViewNode<Key>>;

pub trait TreeViewNode<Key: Hash + Eq + Debug> {
    fn id(&self) -> &Key;
    fn label(&self) -> String;
    fn is_leaf(&self) -> bool;

    fn num_child(&self) -> (bool, usize);
    fn get_child(&self, idx: usize) -> Option<ChildRc<Key>>;
    fn get_child_by_key(&self, key: &Key) -> Option<ChildRc<Key>>;

    fn is_complete(&self) -> bool;

    fn children(&self) -> (bool, Box<dyn Iterator<Item=ChildRc<Key>>>);
}

impl<Key: Hash + Eq + Debug> std::fmt::Debug for dyn TreeViewNode<Key> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeViewNode({:?})", self.id())
    }
}