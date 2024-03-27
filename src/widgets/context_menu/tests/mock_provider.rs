use std::borrow::Cow;

use crate::primitives::tree::tree_node::TreeNode;

#[derive(Debug, Clone)]
pub struct MockNestedMenuItem {
    pub name: String,
    pub children: Vec<MockNestedMenuItem>,
}

impl TreeNode<String> for MockNestedMenuItem {
    fn id(&self) -> &String {
        &self.name
    }

    fn label(&self) -> Cow<str> {
        self.name.as_str().into()
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.children.clone().into_iter())
    }

    fn is_complete(&self) -> bool {
        true
    }
}

pub fn get_mock_data_set_1() -> MockNestedMenuItem {
    MockNestedMenuItem {
        name: "menu1".to_string(),
        children: vec![
            MockNestedMenuItem {
                name: "option1".to_string(),
                children: vec![],
            },
            MockNestedMenuItem {
                name: "option2".to_string(),
                children: vec![],
            },
            MockNestedMenuItem {
                name: "submenu".to_string(),
                children: vec![
                    MockNestedMenuItem {
                        name: "child1".to_string(),
                        children: vec![],
                    },
                    MockNestedMenuItem {
                        name: "child2".to_string(),
                        children: vec![],
                    },
                ],
            },
        ],
    }
}

pub fn get_mock_data_set_2() -> MockNestedMenuItem {
    MockNestedMenuItem {
        name: "root1".to_string(),
        children: vec![
            MockNestedMenuItem {
                name: "option1".to_string(),
                children: vec![],
            },
            MockNestedMenuItem {
                name: "subtree1".to_string(),
                children: vec![MockNestedMenuItem {
                    name: "subsubtree1".to_string(),
                    children: vec![
                        MockNestedMenuItem {
                            name: "subsubtree1child1".to_string(),
                            children: vec![],
                        },
                        MockNestedMenuItem {
                            name: "subsubtree1child2".to_string(),
                            children: vec![],
                        },
                    ],
                }],
            },
            MockNestedMenuItem {
                name: "option2".to_string(),
                children: vec![],
            },
            MockNestedMenuItem {
                name: "subtree2".to_string(),
                children: vec![
                    MockNestedMenuItem {
                        name: "subsubtree2".to_string(),
                        children: vec![MockNestedMenuItem {
                            name: "subsubtree2child1".to_string(),
                            children: vec![],
                        }],
                    },
                    MockNestedMenuItem {
                        name: "subtree2child1".to_string(),
                        children: vec![],
                    },
                    MockNestedMenuItem {
                        name: "subtree2child2".to_string(),
                        children: vec![],
                    },
                ],
            },
        ],
    }
}
