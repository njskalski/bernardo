use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::rc::Rc;

// Keep it lightweight. It is expected to be implemented by Rc<some type>
pub trait TreeViewNode<Key: Hash + Eq + Debug>: Clone + Debug {
    fn id(&self) -> &Key;
    fn label(&self) -> String;
    fn is_leaf(&self) -> bool;

    fn num_child(&self) -> (bool, usize);
    fn get_child(&self, idx: usize) -> Option<Self>;

    fn is_complete(&self) -> bool;
}