use std::fmt::{Debug, Formatter};
use std::hash::Hash;

pub trait TreeViewNode<Key: Hash + Eq + Debug> {
    fn id(&self) -> &Key;
    fn label(&self) -> String;
    fn children(&self) -> Box<dyn std::iter::Iterator<Item=Box<dyn TreeViewNode<Key>>> + '_>;
    fn is_leaf(&self) -> bool;
}

impl<Key: Hash + Eq + Debug> std::fmt::Debug for dyn TreeViewNode<Key> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeViewNode({:?})", self.id())
    }
}
