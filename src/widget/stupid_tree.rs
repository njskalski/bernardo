use std::borrow::Borrow;
use std::ops::Deref;
use std::process::id;
use std::rc::Rc;

use crate::widgets::tree_view::tree_view_node::{ChildRc, TreeViewNode};

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub struct StupidTree {
    id: usize,
    children: Vec<Rc<StupidTree>>,
}

impl StupidTree {
    pub fn new(id: usize, children: Vec<StupidTree>) -> Self {
        StupidTree {
            id,
            children: children.into_iter().map(|c| Rc::new(c)).collect(),
        }
    }
}

impl AsRef<StupidTree> for StupidTree {
    fn as_ref(&self) -> &StupidTree {
        self
    }
}

impl TreeViewNode<usize> for StupidTree {
    fn id(&self) -> &usize {
        &self.id
    }

    fn label(&self) -> String {
        format!("StupidTree {}", self.id)
    }


    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn num_child(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, idx: usize) -> ChildRc<usize> {
        self.children[idx].clone()
    }

    fn get_child_by_key(&self, key: &usize) -> Option<ChildRc<usize>> {
        for child in self.children.iter() {
            if child.id() == key {
                return Some(child.clone());
            };
        };
        return None
    }

    fn has_child(&self, key: &usize) -> bool {
        for c in self.children.iter() {
            if c.id() == key {
                return true;
            }
        }

        false
    }

    fn todo_update_cache(&self) {}
}

pub fn get_stupid_tree() -> Rc<dyn TreeViewNode<usize>> {
    let mut stupid_subtree: Vec<StupidTree> = vec![];

    for i in 0..100 {
        stupid_subtree.push(
            StupidTree::new(40000 + i, vec![])
        );
    }

    let res = StupidTree::new(
        0,
        vec![
            StupidTree::new(
                1,
                vec![
                    StupidTree::new(10001, vec![]),
                    StupidTree::new(10002, vec![]),
                ],
            ),
            StupidTree::new(
                2,
                vec![
                    StupidTree::new(20001, vec![]),
                    StupidTree::new(20002, vec![]),
                    StupidTree::new(20003, vec![StupidTree::new(2000301, vec![])]),
                ],
            ),
            StupidTree::new(4, stupid_subtree),
        ],
    );

    Rc::new(res)
}
