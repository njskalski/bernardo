/*
This iterator implements depth-first-order using a double ended queue to emulate recursion,
so I don't have to fight borrow-checker, that seem hard to marry with lazy instantiation.

I got this idea in Zurich Operahouse, watching some ballet. Creativity sprouts from boredom.
 */
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use crate::io::keys::Key;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

type TreeItFilter = fn(&TreeViewNode<Key>) -> bool;
type QueueType<Item> = Rc<Item>;

// tu nie trzeba pogrzebacza, tu trzeba pogrzebu.

pub struct TreeIt<'a, Key: Hash + Eq + Debug, Item: TreeViewNode<Key>> {
    queue: Vec<(u16, QueueType<Item>)>,
    expanded: &'a HashSet<Key>,
}

impl<'a, Key: Hash + Eq + Debug + Clone, Item: TreeViewNode<Key>> TreeIt<'a, Key, Item> {
    pub fn new(root: &Rc<Item>, expanded: &'a HashSet<Key>) -> TreeIt<'a, Key, Item> {
        let mut queue: Vec<(u16, QueueType<Key>)> = Vec::new();

        queue.push((0, root.clone()));

        TreeIt {
            queue,
            expanded,
        }
    }
}

impl<'a, Key: Hash + Eq + Debug + Clone, Item: TreeViewNode<Key>> Iterator for TreeIt<'a, Key, Item> {
    type Item = (u16, Rc<Item>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.queue.is_empty() == false {
            let head = self.queue.pop().unwrap();
            let (depth, node_ref) = head;

            // If it's expanded, I have to throw all children on the stack.
            if self.expanded.contains(node_ref.id()) {
                for idx in (0..node_ref.num_child()).rev() {
                    let item = node_ref.get_child(idx);
                    self.queue.push(
                        (depth + 1, item)
                    );
                }
            }

            return Some((depth, node_ref))
        }

        None
    }
}
