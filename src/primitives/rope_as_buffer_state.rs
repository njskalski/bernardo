/*
This is a simplified BufferState used for tests.
 */

use log::error;
use ropey::iter::{Chars, Chunks};
use ropey::Rope;

use crate::text::text_buffer::{LinesIter, TextBuffer};

impl TextBuffer for Rope {
    fn byte_to_char(&self, byte_idx: usize) -> Option<usize> {
        self.try_byte_to_char(byte_idx).ok()
    }

    fn char_at(&self, char_idx: usize) -> Option<char> {
        self.get_char(char_idx)
    }

    fn char_to_byte(&self, char_idx: usize) -> Option<usize> {
        self.try_char_to_byte(char_idx).ok()
    }

    fn char_to_line(&self, char_idx: usize) -> Option<usize> {
        self.try_char_to_line(char_idx).ok()
    }

    fn chars(&self) -> Chars {
        ropey::Rope::chars(self)
    }

    fn chunks(&self) -> Chunks {
        ropey::Rope::chunks(self)
    }

    fn insert_block(&mut self, char_idx: usize, block: &str) -> bool {
        self.try_insert(char_idx, block).is_ok()
    }

    fn insert_char(&mut self, char_idx: usize, ch: char) -> bool {
        self.try_insert_char(char_idx, ch).is_ok()
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn len_bytes(&self) -> usize {
        self.len_bytes()
    }

    fn len_chars(&self) -> usize {
        self.len_chars()
    }

    fn len_lines(&self) -> usize {
        self.len_lines()
    }

    fn lines(&self) -> LinesIter {
        LinesIter::new(self.chars())
    }

    fn line_to_char(&self, line_idx: usize) -> Option<usize> {
        self.try_line_to_char(line_idx).ok()
    }

    fn remove(&mut self, char_idx_begin: usize, char_idx_end: usize) -> bool {
        if !(char_idx_end > char_idx_begin) {
            error!("char_idx >= char_idx_begin ( {} >= {} )", char_idx_end, char_idx_begin);
            return false;
        }

        self.try_remove(char_idx_begin..char_idx_end).is_ok()
    }

    fn is_saved(&self) -> bool {
        false
    }

    fn mark_as_saved(&mut self) {
        error!("rope buffer cannot be marked as saved");
        debug_assert!(false);
    }
}
