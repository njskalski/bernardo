// this is for test only

use ropey::Rope;

use crate::text::buffer::Buffer;

impl Buffer for Rope {
    fn len_lines(&self) -> usize {
        self.len_lines()
    }

    fn lines(&self) -> Box<dyn Iterator<Item=&str> + '_> {
        Box::new(self.lines().map(|f| f.as_str().unwrap_or("")).into_iter())
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn len_chars(&self) -> usize {
        self.len_chars()
    }

    fn char_to_line(&self, char_idx: usize) -> Option<usize> {
        match self.try_char_to_line(char_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    fn line_to_char(&self, line_idx: usize) -> Option<usize> {
        match self.try_line_to_char(line_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None
        }
    }

    fn insert_char(&mut self, char_idx: usize, ch: char) -> bool {
        match self.try_insert_char(char_idx, ch) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn remove(&mut self, char_idx_begin: usize, char_idx_end: usize) -> bool {
        if char_idx_end >= char_idx_begin {
            return false;
        }

        match self.try_remove(char_idx_begin..char_idx_end) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn char_at(&self, char_idx: usize) -> Option<char> {
        self.get_char(char_idx)
    }
}