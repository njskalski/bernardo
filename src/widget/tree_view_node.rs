use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;

pub trait TreeViewNode<Key: Hash + Eq + Debug> {
    fn id(&self) -> &Key;
    fn label(&self) -> String;
    fn children<'a>(&'a self) -> Box<(dyn Iterator<Item=&'a (dyn TreeViewNode<Key> + 'a)> + 'a)>;
    fn is_leaf(&self) -> bool;

    fn as_generic(&self) -> &dyn TreeViewNode<Key> {
        self
    }
}

impl<Key: Hash + Eq + Debug> std::fmt::Debug for dyn TreeViewNode<Key> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeViewNode({:?})", self.id())
    }
}
