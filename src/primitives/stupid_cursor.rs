use std::cmp::Ordering;

use log::debug;
use log::error;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::cursor::cursor::Cursor;
use crate::cursor::cursor::Selection;
use crate::primitives::xy::XY;
use crate::text::text_buffer::TextBuffer;
use crate::unpack_or;

/*
This is a completely useless variant of cursor, that is used only because LSP uses it.
It does not easily convert neither to byte offset NOR char idx NOR screen pos, because it does not
mind multi-column chars.
 */
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StupidCursor {
    // Zero based
    pub char_idx_0b: u32,
    // Zero based
    pub line_0b: u32,
}

// TODO fuzzy some test!

impl StupidCursor {
    pub fn new(char_idx: u32, line: u32) -> StupidCursor {
        StupidCursor {
            char_idx_0b: char_idx,
            line_0b: line,
        }
    }

    pub fn from_real_cursor(buffer: &dyn TextBuffer, cursor: Cursor) -> Result<StupidCursor, ()> {
        // TODO I did not implement PositionEncodingKind, so I am not sure if "offset" is utf-8 or byte or soccer fields, or whatever unit of length Americans use now
        let line = match buffer.char_to_line(cursor.a) {
            None => {
                error!("failed casting cursor to LSP cursor - no line for .a");
                return Err(());
            }
            Some(line) => line,
        };

        let begin_line = match buffer.line_to_char(line as usize) {
            None => {
                error!("failed casting cursor to LSP cursor - failed to find line begin");
                return Err(());
            }
            Some(begin) => begin,
        };

        if begin_line > cursor.a {
            error!("failed casting cursor to LSP cursor - line begin > pos in the same line?!");
            return Err(());
        }

        let col = cursor.a - begin_line;

        Ok(StupidCursor {
            line_0b: line as u32,
            char_idx_0b: col as u32,
        })
    }

    pub fn to_xy(&self, rope: &dyn TextBuffer) -> Option<XY> {
        if rope.len_lines() <= self.line_0b as usize {
            debug!("StupidCursor.line {} > {} rope.lines().count", self.line_0b, rope.len_lines());
            return None;
        }
        if self.line_0b >= u16::MAX as u32 {
            debug!("StupidCursor.line {} > u16::MAX", self.line_0b);
            return None;
        }

        let mut x = 0 as u16;
        for g in rope.get_line(self.line_0b as usize) {
            x += g.width() as u16;
        }

        Some(XY::new(x, self.line_0b as u16))
    }

    pub fn to_real_cursor(&self, buffer: &dyn TextBuffer) -> Option<Cursor> {
        let line_begin_char = unpack_or!(buffer.line_to_char(self.line_0b as usize), None, "can't cast stupid cursor to real cursor: not enough lines");
        let candidate = line_begin_char + self.char_idx_0b as usize;
        if let Some(next_line_begin_char) = buffer.line_to_char((self.line_0b + 1) as usize) {
            // I don't know why it works, but it works. So maybe test it, but sharp inequality was failing.
            if candidate <= next_line_begin_char {
                Some(Cursor::new(candidate))
            } else {
                debug!("can't cast stupid cursor to real cursor: not enough chars in given line");
                None
            }
        } else {
            // here we ALLOW character pointing to one-after-buffer
            if candidate <= buffer.len_chars() {
                Some(Cursor::new(candidate))
            } else {
                debug!("can't cast stupid cursor to real cursor: not enough chars in last line");
                None
            }
        }
    }

    pub fn to_real_cursor_range(range: (StupidCursor, StupidCursor), buffer: &dyn TextBuffer) -> Option<Selection> {
        let first = range.0.to_real_cursor(buffer)?;
        let second = range.1.to_real_cursor(buffer)?;
        if first >= second {
            error!("failed to convert stupid cursor range to real cursor - first = {:?} >= {:?} = second", first, second);
            return None;
        }

        Some(Selection::new(first.a, second.a))
    }

    pub fn is_between(&self, left_inclusive: StupidCursor, right_exclusive: StupidCursor) -> bool {
        if left_inclusive >= right_exclusive {
            error!("stupid cursor {:?} can't be within deformed range {:?} {:?}", self, left_inclusive, right_exclusive);
            return false;
        }

        left_inclusive <= *self && *self < right_exclusive
    }
}

impl PartialOrd<Self> for StupidCursor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StupidCursor {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp1 = self.line_0b.cmp(&other.line_0b);
        if cmp1 != Ordering::Equal {
            cmp1
        } else {
            self.char_idx_0b.cmp(&other.char_idx_0b)
        }
    }
}

