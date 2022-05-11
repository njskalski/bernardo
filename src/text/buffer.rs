use std::iter;
use std::iter::Peekable;
use std::marker::PhantomData;
use ropey::iter::Chars;
use streaming_iterator::StreamingIterator;
use crate::tsw::lang_id::LangId;

//TODO create tests for undo/redo/set milestone

pub trait Buffer {
    fn len_lines(&self) -> usize;

    fn new_lines(&self) -> LinesIter;

    fn is_editable(&self) -> bool;

    fn len_chars(&self) -> usize;

    /*
    This function will succeed with idx one beyond the limit, so with char_idx == len_chars().
    It's a piece of Ropey semantics I will not remove now.
     */
    fn char_to_line(&self, char_idx: usize) -> Option<usize>;
    fn line_to_char(&self, line_idx: usize) -> Option<usize>;

    fn insert_char(&mut self, char_idx: usize, ch: char) -> bool;
    fn insert_block(&mut self, char_idx: usize, block: &str) -> bool;
    fn remove(&mut self, char_idx_begin: usize, char_idx_end: usize) -> bool;

    fn char_at(&self, char_idx: usize) -> Option<char>;

    fn chars(&self) -> Chars;

    fn char_to_kind(&self, _char_idx: usize) -> Option<&str> { None }

    fn callback_for_parser<'a>(&'a self) -> Box<dyn FnMut(usize, tree_sitter::Point) -> &'a [u8] + 'a>;

    fn try_parse(&mut self, _lang_id: LangId) -> bool { false }

    fn can_undo(&self) -> bool { false }
    fn can_redo(&self) -> bool { false }

    /*
    In order for redo to work, undo implies "set_milestone"
     */
    fn undo(&mut self) -> bool { false }
    fn redo(&mut self) -> bool { false }

    /*
    This creates new milestone to undo/redo. The reason for it is that potentially multiple edits inform a single milestone.
    Returns false only if buffer have not changed since last milestone.
     */
    fn set_milestone(&mut self) -> bool { false }
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

pub fn buffer_to_string(b: &dyn Buffer) -> String {
    let mut output = String::new();

    let mut line_it = b.new_lines();
    while let Some(line) = line_it.next() {
        output += line.as_str()
    }

    output
}