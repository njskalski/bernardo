use std::ops::RangeBounds;
use std::rc::Rc;

use log::{error, warn};
use ropey::Rope;

use crate::text::buffer::Buffer;

#[derive(Clone, Debug)]
pub struct BufferState {
    text: Rope,

    history: Vec<Rope>,
    forward_history: Vec<Rope>,
}

impl BufferState {
    pub fn new() -> BufferState {
        BufferState {
            text: Rope::new(),
            history: vec![],
            forward_history: vec![],
        }
    }

    pub fn with_text(self, text: &str) -> BufferState {
        BufferState {
            text: Rope::from_str(text),
            ..self
        }
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
}

impl Buffer for BufferState {
    fn len_lines(&self) -> usize {
        self.text.len_lines()
    }
    fn lines(&self) -> Box<dyn std::iter::Iterator<Item=&str> + '_> {
        Box::new(self.text.lines().map(|line| line.as_str().unwrap()))
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
}