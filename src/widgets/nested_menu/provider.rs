use std::fmt::Debug;
use crate::primitives::printable::Printable;

pub enum NodeType {
    Leaf,
    Branch
}

pub trait NestedMenuItem : Debug{
    fn display_name(&self) -> impl Printable;

    fn node_type(&self) -> NodeType;

    fn children(&self) -> impl Iterator<Item=&Self>;
}

pub trait NestedMenuProvider<Item : NestedMenuItem> {}