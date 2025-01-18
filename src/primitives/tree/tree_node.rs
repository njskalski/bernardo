use crate::io::keys;
use crate::primitives::tree::filter_policy::FilterPolicy;
use crate::promise::streaming_promise::StreamingPromise;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

// Keep it lightweight. It is expected to be implemented by Rc<some type>
pub trait TreeNode<Key: Hash + Eq + Debug>: Clone + Debug {
    fn id(&self) -> &Key;
    fn label(&self) -> Cow<str>;

    fn keyboard_shortcut(&self) -> Option<keys::Key> {
        None
    }
    fn is_leaf(&self) -> bool;

    fn child_iter(&self) -> Box<dyn Iterator<Item = Self> + '_>;

    fn is_complete(&self) -> bool;

    fn is_single_subtree(&self) -> bool {
        if self.is_leaf() {
            return false;
        }

        for child in self.child_iter() {
            if !child.is_leaf() {
                return false;
            }
        }

        true
    }

    fn get_streaming_promise_instead_of_iterator(
        &self,
        filter_op: Option<(FilterRef<Self>, FilterPolicy)>,
        expanded_op: Option<HashSet<Key>>,
    ) -> Option<Box<dyn StreamingPromise<(u16, Self)>>> {
        None
    }
}

pub trait TreeItFilter<Node>: Send + Sync + 'static {
    fn call(&self, node: &Node) -> bool;

    fn arc_box(self) -> Arc<Box<dyn TreeItFilter<Node> + Send + Sync + 'static>>
    where
        Self: Sized,
    {
        Arc::new(Box::new(self))
    }
}

pub struct ClosureFilter<Node> {
    function: Box<dyn for<'a> Fn(&'a Node) -> bool + Send + Sync + 'static>,
}

impl<Node> ClosureFilter<Node> {
    pub fn new<F: for<'a> Fn(&'a Node) -> bool + Send + Sync + 'static>(f: F) -> Self {
        ClosureFilter { function: Box::new(f) }
    }
}

impl<Node: 'static> TreeItFilter<Node> for ClosureFilter<Node> {
    fn call(&self, node: &Node) -> bool {
        (self.function)(node)
    }
}

pub type FilterRef<Node> = Arc<Box<dyn TreeItFilter<Node> + Send + Sync + 'static>>;
