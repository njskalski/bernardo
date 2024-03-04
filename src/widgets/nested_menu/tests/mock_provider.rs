use crate::primitives::printable::Printable;
use crate::widgets::nested_menu::provider::{NestedMenuItem, NestedMenuProvider, NodeType};

#[derive(Debug)]
pub struct MockNestedMenuItem {
    pub name : String,
    pub children : Vec<MockNestedMenuItem>
}

impl NestedMenuItem for MockNestedMenuItem {
    fn display_name(&self) -> impl Printable {
        self.name.as_str()
    }

    fn node_type(&self) -> NodeType {
        if self.children.is_empty() {
            NodeType::Leaf
        } else {
            NodeType::Branch
        }
    }

    fn children(&self) -> impl Iterator<Item=&Self> {
        self.children.iter()
    }
}

struct MockNestedMenuProvider {

}

impl MockNestedMenuProvider {

}

impl NestedMenuProvider<MockNestedMenuItem> for MockNestedMenuProvider {

}