use log::debug;
use ropey::iter::{Bytes, Chunks};
use ropey::{iter, RopeSlice};
use std::iter::Peekable;
use std::ops::Deref;

use tree_sitter::{Node, TextProvider};

pub struct RopeWrapper<'a>(pub &'a ropey::Rope);

pub struct WrappedChunks<'a>(ropey::iter::Chunks<'a>);
pub static EMPTY_LIST: [u8; 0] = [];

impl<'a> Iterator for WrappedChunks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> TextProvider<&'a str> for RopeWrapper<'a> {
    type I = WrappedChunks<'a>;

    fn text(&mut self, node: Node<'_>) -> Self::I {
        let char_begin = self.0.byte_to_char(node.start_byte());
        let char_end = self.0.byte_to_char(node.end_byte());

        debug!(
            "rope_wrapper reads [{}:{}) from node {:?} = [{}]",
            char_begin,
            char_end,
            node,
            self.0.slice(char_begin..char_end)
        );

        WrappedChunks(self.0.slice(char_begin..char_end).chunks())
    }
}
