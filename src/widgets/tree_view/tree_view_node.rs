use std::any::Any;
use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;

pub type ChildRc<Key> = Rc<dyn TreeViewNode<Key>>;

pub trait TreeViewNode<Key: Hash + Eq + Debug> {
    fn id(&self) -> &Key;
    fn label(&self) -> String;
    fn is_leaf(&self) -> bool;

    fn num_child(&self) -> usize;
    fn get_child(&self, idx: usize) -> ChildRc<Key>;
    fn get_child_by_key(&self, key: &Key) -> Option<ChildRc<Key>>;

    fn has_child(&self, key: &Key) -> bool;

    // I am not sure if this should be there, or the filesystem provider should just re-issue
    // entire tree.
    fn todo_update_cache(&self);
}

impl<Key: Hash + Eq + Debug> std::fmt::Debug for dyn TreeViewNode<Key> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeViewNode({:?})", self.id())
    }
}
