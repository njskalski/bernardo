use std::fmt::{Debug, Formatter};

use ropey::iter::{Chars, Chunks};
use streaming_iterator::StreamingIterator;

use crate::experiments::deref_str::DerefStr;
use crate::tsw::lang_id::LangId;

//TODO create tests for undo/redo/set milestone

pub trait TextBuffer: ToString {
    fn len_lines(&self) -> usize;

    fn lines(&self) -> LinesIter;

    fn is_editable(&self) -> bool;

    fn len_chars(&self) -> usize;
    fn len_bytes(&self) -> usize;

    /*
    This function will succeed with idx one beyond the limit, so with char_idx == len_chars().
    It's a piece of Ropey semantics I will not remove now.
     */
    fn char_to_line(&self, char_idx: usize) -> Option<usize>;
    fn line_to_char(&self, line_idx: usize) -> Option<usize>;

    fn byte_to_char(&self, byte_idx: usize) -> Option<usize>;
    fn char_to_byte(&self, char_idx: usize) -> Option<usize>;

    fn insert_char(&mut self, char_idx: usize, ch: char) -> bool;
    fn insert_block(&mut self, char_idx: usize, block: &str) -> bool;
    fn remove(&mut self, char_idx_begin: usize, char_idx_end: usize) -> bool;

    fn char_at(&self, char_idx: usize) -> Option<char>;

    fn chars(&self) -> Chars;
    fn chunks(&self) -> Chunks;

    fn char_to_kind(&self, _char_idx: usize) -> Option<&str> { None }

    fn callback_for_parser<'a>(&'a self) -> Box<dyn FnMut(usize, tree_sitter::Point) -> &'a [u8] + 'a>;

    fn try_parse(&mut self, _lang_id: LangId) -> bool { false }

    fn can_undo(&self) -> bool { false }
    fn can_redo(&self) -> bool { false }
    fn undo(&mut self) -> bool { false }
    fn redo(&mut self) -> bool { false }

    fn tab_width(&self) -> usize { 4 }
}

pub struct LinesIter<'a> {
    line: String,
    iter: Box<dyn Iterator<Item=char> + 'a>,
    done: bool,
}

impl<'a> LinesIter<'a> {
    pub fn new<T: Iterator<Item=char> + 'a>(iter: T) -> Self {
        LinesIter {
            line: String::new(),
            iter: Box::new(iter),
            done: false,
        }
    }
}

impl<'a> StreamingIterator for LinesIter<'a> {
    type Item = String;

    fn advance(&mut self) {
        self.line.clear();

        loop {
            if let Some(char) = self.iter.next() {
                self.line.push(char);

                if char == '\n' {
                    break;
                }
            } else {
                self.done = true;
                break;
            }
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.done && self.line.is_empty() {
            None
        } else {
            Some(&self.line)
        }
    }
}

impl Debug for dyn TextBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut chunks = self.chunks();
        if let Some(chunk) = chunks.next() {
            f.write_str(chunk.as_ref_str())?;
        }

        Ok(())
    }
}