use std::borrow::Borrow;
use std::ops::Range;
use std::string::String;

use log::{debug, error, warn};
use ropey::iter::Lines;
use ropey::Rope;
use tree_sitter::{Tree, TreeCursor};
use unicode_segmentation::UnicodeSegmentation;

use crate::experiments::try_parse::try_parsing_rust;
use crate::text::buffer::Buffer;
use crate::Theme;

#[derive(Clone, Debug)]
pub struct BufferState {
    text: Rope,

    history: Vec<Rope>,
    forward_history: Vec<Rope>,

    todo_parse_tree: Option<tree_sitter::Tree>,
}

impl BufferState {
    pub fn new() -> BufferState {
        BufferState {
            text: Rope::new(),
            history: vec![],
            forward_history: vec![],
            todo_parse_tree: None,
        }
    }

    pub fn with_text_from_string<'a, T: Into<&'a str>>(self, text: T) -> BufferState {
        let mut res = BufferState {
            text: Rope::from_str(text.into()),
            ..self
        };

        match try_parsing_rust(res.text.to_string().as_str()) {
            None => {}
            Some(tree) => {
                // todo_dump_tree(&tree);
                res.todo_parse_tree = Some(tree);
            }
        };

        res
    }

    fn clone_top(&mut self) {
        self.history.push(self.text.clone());
    }

    fn after_change(&mut self) {
        self.forward_history.clear();
    }

    pub fn prev(&mut self) -> bool {
        match self.history.pop() {
            None => false,
            Some(r) => {
                self.forward_history.push(self.text.clone());
                self.text = r;
                true
            }
        }
    }

    pub fn next(&mut self) -> bool {
        match self.forward_history.pop() {
            None => false,
            Some(r) => {
                self.history.push(self.text.clone());
                self.text = r;
                true
            }
        }
    }

    pub fn char_to_kind(&self, char_idx: usize) -> Option<&str> {
        let byte_idx = match self.text.try_char_to_byte(char_idx) {
            Ok(idx) => idx,
            _ => return None,
        };

        self.todo_parse_tree.as_ref().map(|tree| {
            tree.root_node().descendant_for_byte_range(byte_idx, byte_idx)
        }).flatten().map(|node| {
            node.kind()
        })
    }
}

impl Buffer for BufferState {
    fn len_lines(&self) -> usize {
        self.text.len_lines()
    }
    fn lines(&self) -> Box<dyn Iterator<Item=String> + '_> {
        // TODO this will fail for large files
        Box::new(self.text.lines().map(|f| f.to_string()))
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    fn char_to_line(&self, char_idx: usize) -> Option<usize> {
        match self.text.try_char_to_line(char_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    fn line_to_char(&self, line_idx: usize) -> Option<usize> {
        match self.text.try_line_to_char(line_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    fn insert_char(&mut self, char_idx: usize, ch: char) -> bool {
        match self.text.try_insert_char(char_idx, ch) {
            Ok(_) => true,
            Err(e) => {
                warn!("failed inserting char {} because {}", char_idx, e);
                false
            }
        }
    }

    fn remove(&mut self, char_idx_begin: usize, char_idx_end: usize) -> bool {
        if !(char_idx_end > char_idx_begin) {
            error!("requested removal of improper range ({}, {})", char_idx_begin, char_idx_end);
            return false;
        }

        match self.text.try_remove(char_idx_begin..char_idx_end) {
            Ok(_) => true,
            Err(e) => {
                warn!("failed removing char {:?}-{:?} because {}", char_idx_begin, char_idx_end, e);
                false
            }
        }
    }

    fn char_at(&self, char_idx: usize) -> Option<char> {
        self.text.char_at(char_idx)
    }
}