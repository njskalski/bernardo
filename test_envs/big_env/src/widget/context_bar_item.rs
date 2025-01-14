use std::borrow::Cow;
use std::fmt::{Debug, Formatter};

use crate::io::keys::Key;
use crate::primitives::tree::tree_node::TreeNode;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widgets::editor_widget::msg::EditorWidgetMsg;

pub type Action = fn() -> Box<dyn AnyMsg>;
pub type TreeKey = Cow<'static, str>;

#[derive(Clone)]
pub enum NodeType {
    // by key here I mean "keycode" not "hash key"
    Leaf { action: Action, key: Option<Key> },
    InternalNode(Vec<ContextBarItem>),
}

#[derive(Clone)]
pub struct ContextBarItem {
    title: Cow<'static, str>,
    node_type: NodeType,
    depth: usize,
}

impl ContextBarItem {
    pub fn new_internal_node(title: Cow<'static, str>, children: Vec<ContextBarItem>) -> ContextBarItem {
        ContextBarItem {
            title,
            node_type: NodeType::InternalNode(children),
            depth: 0,
        }
    }

    pub fn new_leaf_node(title: Cow<'static, str>, action: Action, key_op: Option<Key>) -> ContextBarItem {
        ContextBarItem {
            title,
            node_type: NodeType::Leaf { action, key: key_op },
            depth: 0,
        }
    }

    pub fn set_depth_recursively(&mut self, depth: usize) {
        self.depth = depth;

        for child in self.child_iter_mut() {
            child.set_depth_recursively(depth);
        }
    }

    fn child_iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Self> + '_> {
        match &mut self.node_type {
            NodeType::Leaf { .. } => Box::new(std::iter::Empty::default()),
            NodeType::InternalNode(items) => Box::new(items.iter_mut()),
        }
    }

    pub fn get_depth(&self) -> usize {
        self.depth
    }

    pub const GO_TO_DEFINITION: ContextBarItem = ContextBarItem {
        title: Cow::Borrowed("go to definition"),
        node_type: NodeType::Leaf {
            action: || EditorWidgetMsg::GoToDefinition.boxed(),
            key: None,
        },
        depth: 0,
    };
    pub const REFORMAT_FILE: ContextBarItem = ContextBarItem {
        title: Cow::Borrowed("reformat file"),
        node_type: NodeType::Leaf {
            action: || EditorWidgetMsg::Reformat.boxed(),
            key: None,
        },
        depth: 0,
    };
    pub const SHOW_USAGES: ContextBarItem = ContextBarItem {
        title: Cow::Borrowed("show usages"),
        node_type: NodeType::Leaf {
            action: || EditorWidgetMsg::ShowUsages.boxed(),
            key: None,
        },
        depth: 0,
    };
}

impl Debug for ContextBarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let desc = match &self.node_type {
            NodeType::Leaf { .. } => "leaf".to_string(),
            NodeType::InternalNode(vec) => format!("node({})", vec.len()),
        };

        write!(f, "({} {})", desc, self.title)
    }
}

impl TreeNode<TreeKey> for ContextBarItem {
    fn id(&self) -> &TreeKey {
        &self.title
    }

    fn label(&self) -> Cow<str> {
        self.title.clone()
    }

    fn keyboard_shortcut(&self) -> Option<Key> {
        match self.node_type {
            NodeType::InternalNode(_) => None,
            NodeType::Leaf { action, key } => key,
        }
    }

    fn is_leaf(&self) -> bool {
        match self.node_type {
            NodeType::Leaf { .. } => true,
            NodeType::InternalNode(_) => false,
        }
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item = Self>> {
        match &self.node_type {
            NodeType::Leaf { .. } => Box::new(std::iter::Empty::default()),
            NodeType::InternalNode(items) => Box::new(items.clone().into_iter()),
        }
    }

    fn is_complete(&self) -> bool {
        true
    }
}

impl ContextBarItem {
    pub fn on_hit(&self) -> Option<Box<dyn AnyMsg>> {
        match self.node_type {
            NodeType::Leaf { action, key } => Some(action()),
            NodeType::InternalNode(_) => None,
        }
    }
}
