use std::fmt::{Debug, Formatter};

use log::{error, warn};
use ropey::iter::{Chars, Chunks};
use streaming_iterator::StreamingIterator;

use crate::cursor::cursor::{Cursor, Selection};
use crate::primitives::stupid_cursor::StupidCursor;
use crate::primitives::xy::XY;
use crate::tsw::lang_id::LangId;
use crate::unpack_or_e;
//TODO create tests for undo/redo/set milestone

pub trait TextBuffer: ToString {
    fn byte_to_char(&self, byte_idx: usize) -> Option<usize>;

    fn char_at(&self, char_idx: usize) -> Option<char>;
    fn char_to_byte(&self, char_idx: usize) -> Option<usize>;
    /*
    This function will succeed with idx one beyond the limit, so with char_idx == len_chars().
    It's a piece of Ropey semantics I will not remove now.
     */
    fn char_to_line(&self, char_idx: usize) -> Option<usize>;
    fn chars(&self) -> Chars;
    fn chunks(&self) -> Chunks;

    // second parameter is whether all chars in selection were found
    fn get_selected_chars(&self, selection: Selection) -> (Option<String>, bool) {
        if selection.b >= self.len_chars() {
            return (None, false);
        }

        let mut s: String = String::new();

        for char_idx in selection.b..selection.e {
            if let Some(c) = self.char_at(char_idx) {
                s.push(c);
            } else {
                return (Some(s), false);
            }
        }

        (Some(s), true)
    }

    fn insert_block(&mut self, char_idx: usize, block: &str) -> bool;
    fn insert_char(&mut self, char_idx: usize, ch: char) -> bool;
    fn is_editable(&self) -> bool;
    fn len_bytes(&self) -> usize;
    fn len_chars(&self) -> usize;
    fn len_lines(&self) -> usize;
    fn lines(&self) -> LinesIter;
    fn line_to_char(&self, line_idx: usize) -> Option<usize>;
    fn redo(&mut self) -> bool {
        false
    }
    fn remove(&mut self, char_idx_begin: usize, char_idx_end: usize) -> bool;
    fn try_parse(&mut self, _lang_id: LangId) -> bool {
        false
    }
    fn undo(&mut self) -> bool {
        false
    }

    // TODO test
    fn char_idx_to_xy(&self, char_idx: usize) -> Option<XY> {
        if self.len_chars() < char_idx {
            return None;
        }

        let line_idx_0b = self.char_to_line(char_idx).unwrap();
        let char_idx_in_line_0b = char_idx - line_idx_0b;

        if line_idx_0b > u16::MAX as usize {
            error!("line index too high");
            return None;
        }

        if char_idx_in_line_0b > u16::MAX as usize {
            error!("char in line index too high");
            return None;
        }

        Some(XY::new(char_idx_in_line_0b as u16, line_idx_0b as u16))
    }

    // TODO test
    fn get_line(&self, line_idx_0b: usize) -> Option<String> {
        if self.len_lines() <= line_idx_0b {
            return None;
        }

        let line_begin_char_idx_0b = self.line_to_char(line_idx_0b)?;
        let mut result = String::new();
        for char in self.chars().skip(line_begin_char_idx_0b) {
            if char != '\n' {
                result.push(char);
            } else {
                break;
            }
        }

        Some(result)
    }

    fn stupid_cursor_to_cursor(&self, sc1: StupidCursor, sc2: Option<StupidCursor>) -> Option<Cursor> {
        let pos1 = self.line_to_char(sc1.line_0b as usize)? + sc1.char_idx_0b as usize;
        let pos2: Option<usize> = match sc2 {
            None => None,
            Some(sc2) => Some(self.line_to_char(sc2.line_0b as usize)? + sc2.char_idx_0b as usize),
        };

        let mut cursor = Cursor::new(pos1);
        if let Some(pos2) = pos2 {
            if pos1 != pos2 {
                cursor = cursor.with_selection(Selection::new(pos1, pos2));
            } else {
                warn!("weird, malformed, stupid cursor (pos1 == pos2)");
            }
        };

        Some(cursor)
    }

    fn is_saved(&self) -> bool;
    fn mark_as_saved(&mut self);

    fn char_idx_to_begin_line_char_idx(&self, char_idx: usize) -> Option<usize> {
        let line_idx = unpack_or_e!(self.char_to_line(char_idx), None, "failed to get line idx");
        let begin_line_char_idx = unpack_or_e!(self.line_to_char(line_idx), None, "failed to get char idx for line idx");
        Some(begin_line_char_idx)
    }

    fn get_tab_width(&self) -> Option<u8> {
        None
    }

    fn get_tabs_expansion_for_line(&self, line_no_0b: usize) -> u16 {
        let mut added_from_tabs: u16 = 0;
        if let Some(tabs_to_spaces) = self.get_tab_width() {
            if let Some(line) = self.get_line(line_no_0b) {
                for char in line.chars() {
                    if char == '\t' {
                        added_from_tabs += tabs_to_spaces as u16;
                    }
                }
            }
        }
        added_from_tabs
    }
}

pub struct LinesIter<'a> {
    line: String,
    iter: Box<dyn Iterator<Item = char> + 'a>,
    done: bool,
}

impl<'a> LinesIter<'a> {
    pub fn new<T: Iterator<Item = char> + 'a>(iter: T) -> Self {
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
            f.write_str(chunk)?;
        }

        Ok(())
    }
}
