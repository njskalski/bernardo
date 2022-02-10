use std::iter;

use crate::experiments::tree_sitter_wrapper::LangId;

pub trait Buffer {
    fn len_lines(&self) -> usize;
    fn lines(&self) -> Box<dyn iter::Iterator<Item=String> + '_>;

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

    fn char_to_kind(&self, char_idx: usize) -> Option<&str> { None }

    fn callback_for_parser<'a>(&'a self) -> Box<FnMut(usize, tree_sitter::Point) -> &'a [u8] + 'a>;

    fn try_parse(&mut self, langId: LangId) -> bool { false }
}

pub fn buffer_to_string(b: &dyn Buffer) -> String {
    let mut output = String::new();

    for line in b.lines() {
        output += line.as_str()
    }

    output
}