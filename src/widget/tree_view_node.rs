use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ops::Deref;

pub type ChildrenIt<Key> = Box<(dyn Iterator<Item=Box<dyn Borrow<dyn TreeViewNode<Key>>>>)>;

pub trait TreeViewNode<Key: Hash + Eq + Debug> {
    fn id(&self) -> &Key;
    fn label(&self) -> String;
    fn children(&mut self) -> ChildrenIt<Key>;
    fn is_leaf(&self) -> bool;

    fn as_generic(&self) -> Box<dyn Borrow<TreeViewNode<Key>>> {
        Box::new(self)
    }
}

impl<Key: Hash + Eq + Debug> std::fmt::Debug for dyn TreeViewNode<Key> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeViewNode({:?})", self.id())
    }
}
