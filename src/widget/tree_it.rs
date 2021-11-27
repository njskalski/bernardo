/*
This iterator implements depth-first-order using a double ended queue to emulate recursion,
so I don't have to fight borrow-checker, that seem hard to marry with lazy instantiation.

I got this idea in Zurich Operahouse, watching some ballet. Creativity sprouts from boredom.
 */
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

use either::Either;

use crate::widget::tree_view_node::TreeViewNode;

type QueueType<'a, Key> = &'a dyn AsRef<TreeViewNode<Key>>;

pub struct TreeIt<'a, Key: Hash + Eq + Debug> {
    depth: usize,
    queue: Vec<(u16, QueueType<'a, Key>)>,
    expanded: &'a HashSet<Key>,
}

impl<'a, Key: Hash + Eq + Debug + Clone> TreeIt<'a, Key> {
    pub fn new(root: &'a Box<dyn TreeViewNode<Key>>, expanded: &'a HashSet<Key>) -> TreeIt<'a, Key> {
        let mut queue: Vec<(u16, QueueType<'a, Key>)> = Vec::new();

        queue.push_front((0, Box::new(std::iter::once(root))));

        TreeIt {
            depth: 0,
            queue,
            expanded,
        }
    }
}

impl<'a, Key: Hash + Eq + Debug + Clone> Iterator for TreeIt<'a, Key> {
    type Item = (u16, Box<dyn TreeViewNode<Key>>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.queue.is_empty() == false {
            let head = self.queue.front_mut().unwrap();


            let (depth, iterator) = head;
            match iterator.next() {
                None => {
                    self.queue.pop_front();
                    continue;
                },
                Some(item) => {
                    self.item = Some(item);

                    if self.expanded.contains(item.id()) {
                        self.queue.push_front((*depth + 1, self.item.unwrap().children()));
                    }

                    return Some((*depth, item))
                }
            }
        }

        None
    }
}

