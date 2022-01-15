use std::any::Any;

use crate::primitives::color::Color;
use crate::Theme;

pub struct ParseSymbol<'a> {
    text: &'a str,
}

impl<'a> ParseSymbol<'a> {
    pub fn new(text: &str) -> ParseSymbol {
        ParseSymbol {
            text
        }
    }

    // pub fn from_tree_sitter<'a>(ts_node : &'a tree_sitter::Node) -> ParseSymbol<'a> {
    //     let x = ts_node.
    // }

    pub fn color(&self, theme: &mut Theme) -> Color {
        theme.default_text(false).foreground
    }
}

