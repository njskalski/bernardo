use std::borrow::Cow;

use crate::primitives::tree::tree_node::TreeNode;

#[derive(Debug, Clone)]
pub struct MockTreeItem {
    pub name: String,
    pub children: Vec<MockTreeItem>,
}

impl TreeNode<String> for MockTreeItem {
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

pub fn get_mock_data_set_1() -> MockTreeItem {
    MockTreeItem {
        name: "menu1".to_string(),
        children: vec![
            MockTreeItem {
                name: "option1".to_string(),
                children: vec![],
            },
            MockTreeItem {
                name: "option2".to_string(),
                children: vec![],
            },
            MockTreeItem {
                name: "submenu".to_string(),
                children: vec![
                    MockTreeItem {
                        name: "child1".to_string(),
                        children: vec![],
                    },
                    MockTreeItem {
                        name: "child2".to_string(),
                        children: vec![],
                    },
                ],
            },
        ],
    }
}

pub fn get_mock_data_set_2() -> MockTreeItem {
    MockTreeItem {
        name: "root1".to_string(),
        children: vec![
            MockTreeItem {
                name: "option1".to_string(),
                children: vec![],
            },
            MockTreeItem {
                name: "subtree1".to_string(),
                children: vec![MockTreeItem {
                    name: "subsubtree1".to_string(),
                    children: vec![
                        MockTreeItem {
                            name: "subsubtree1child1".to_string(),
                            children: vec![],
                        },
                        MockTreeItem {
                            name: "subsubtree1child2".to_string(),
                            children: vec![],
                        },
                    ],
                }],
            },
            MockTreeItem {
                name: "option2".to_string(),
                children: vec![],
            },
            MockTreeItem {
                name: "subtree2".to_string(),
                children: vec![
                    MockTreeItem {
                        name: "subsubtree2".to_string(),
                        children: vec![MockTreeItem {
                            name: "subsubtree2child1".to_string(),
                            children: vec![],
                        }],
                    },
                    MockTreeItem {
                        name: "subtree2child1".to_string(),
                        children: vec![],
                    },
                    MockTreeItem {
                        name: "subtree2child2".to_string(),
                        children: vec![],
                    },
                ],
            },
        ],
    }
}

pub fn get_mock_data_set_3() -> MockTreeItem {
    MockTreeItem {
        name: "root1".to_string(),
        children: vec![
            MockTreeItem {
                name: "option1".to_string(),
                children: vec![],
            },
            MockTreeItem {
                name: "subtree1".to_string(),
                children: vec![MockTreeItem {
                    name: "subsubtree1".to_string(),
                    children: vec![
                        MockTreeItem {
                            name: "subsubtree1child1".to_string(),
                            children: vec![],
                        },
                        MockTreeItem {
                            name: ".subsubtree1hiddenchild2".to_string(),
                            children: vec![],
                        },
                    ],
                }],
            },
            MockTreeItem {
                name: ".hiddenoption2".to_string(),
                children: vec![],
            },
            MockTreeItem {
                name: ".hiddensubtree2".to_string(),
                children: vec![
                    MockTreeItem {
                        name: "subsubtree2".to_string(),
                        children: vec![MockTreeItem {
                            name: "subsubtree2child1".to_string(),
                            children: vec![],
                        }],
                    },
                    MockTreeItem {
                        name: "subtree2child1".to_string(),
                        children: vec![],
                    },
                    MockTreeItem {
                        name: "subtree2child2".to_string(),
                        children: vec![],
                    },
                ],
            },
        ],
    }
}
