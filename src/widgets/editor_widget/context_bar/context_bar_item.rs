use std::borrow::Cow;
use std::fmt::Debug;

use crate::primitives::tree::tree_node::TreeNode;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widgets::editor_widget::msg::EditorWidgetMsg;

pub type Action = fn() -> Box<dyn AnyMsg>;
pub type Key = Cow<'static, str>;

#[derive(Debug, Clone)]
pub enum NodeType {
    Leaf(Action),
    InternalNode(Vec<ContextBarItem>),
}

#[derive(Debug, Clone)]
pub struct ContextBarItem {
    title: Cow<'static, str>,
    node_type: NodeType,
}

impl ContextBarItem {
    pub fn new_internal_node(title: Cow<'static, str>, children: Vec<ContextBarItem>) -> ContextBarItem {
        ContextBarItem {
            title,
            node_type: NodeType::InternalNode(children),
        }
    }

    pub const GO_TO_DEFINITION: ContextBarItem = ContextBarItem {
        title: Cow::Borrowed("go to definition"),
        node_type: NodeType::Leaf(|| EditorWidgetMsg::GoToDefinition.boxed()),
    };
    pub const REFORMAT_FILE: ContextBarItem = ContextBarItem {
        title: Cow::Borrowed("reformat file"),
        node_type: NodeType::Leaf(|| EditorWidgetMsg::Reformat.boxed()),
    };
    pub const SHOW_USAGES: ContextBarItem = ContextBarItem {
        title: Cow::Borrowed("show usages"),
        node_type: NodeType::Leaf(|| EditorWidgetMsg::ShowUsages.boxed()),
    };
}

impl TreeNode<Key> for ContextBarItem {
    fn id(&self) -> &Key {
        &self.title
    }

    fn label(&self) -> Cow<str> {
        self.title.clone()
    }

    fn is_leaf(&self) -> bool {
        match self.node_type {
            NodeType::Leaf(_) => true,
            NodeType::InternalNode(_) => false,
        }
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item = Self>> {
        match &self.node_type {
            NodeType::Leaf(_) => Box::new(std::iter::Empty::default()),
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
            NodeType::Leaf(action) => Some(action()),
            NodeType::InternalNode(_) => None,
        }
    }
}
