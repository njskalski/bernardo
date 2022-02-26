use std::rc::Rc;

use crate::widgets::tree_view::tree_view_node::TreeViewNode;

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

    fn num_child(&self) -> (bool, usize) {
        (true, self.children.len())
    }

    fn get_child(&self, idx: usize) -> Option<Rc<Self>> {
        self.children.get(idx).map(|f| f.clone())
    }

    // fn get_child_by_key(&self, key: &usize) -> Option<Rc<Self>> {
    //     for child in self.children.iter() {
    //         if child.id() == key {
    //             return Some(child.clone());
    //         };
    //     };
    //     return None
    // }

    fn is_complete(&self) -> bool {
        true
    }

    // fn children(&self) -> (bool, Box<dyn Iterator<Item=Rc<Self>>>) {
    //     (true, Box::new(self.children.iter()))
    // }

    // fn has_child(&self, key: &usize) -> bool {
    //     for c in self.children.iter() {
    //         if c.id() == key {
    //             return true;
    //         }
    //     }
    //
    //     false
    // }
    //
    // fn todo_update_cache(&self) {}
}

pub fn get_stupid_tree() -> Rc<StupidTree> {
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
            StupidTree::new(4, vec![]),
            StupidTree::new(4, stupid_subtree),
        ],
    );

    Rc::new(res)
}
