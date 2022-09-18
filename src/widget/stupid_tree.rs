use std::borrow::Cow;
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

impl TreeViewNode<usize> for Rc<StupidTree> {
    fn id(&self) -> &usize {
        &self.id
    }

    fn label(&self) -> Cow<str> {
        format!("StupidTree {}", self.id).into()
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item=&Self> + '_> {
        Box::new(self.children.iter())
    }

    fn is_complete(&self) -> bool {
        true
    }
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
