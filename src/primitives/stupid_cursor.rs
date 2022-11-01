use log::debug;
use log::error;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::primitives::cursor_set::Cursor;
use crate::primitives::xy::XY;
use crate::text::text_buffer::TextBuffer;

/*
This is a completely useless variant of cursor, that is used only because LSP uses it.
It does not easily convert neither to char idx NOR screen pos, because it does not mind multi-column
 chars.
 */
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StupidCursor {
    pub char_idx: u32,
    pub line: u32,
}

impl StupidCursor {
    pub fn from_real_cursor(buffer: &dyn TextBuffer, cursor: &Cursor) -> Result<StupidCursor, ()> {
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
            line: line as u32,
            char_idx: col as u32,
        })
    }

    pub fn to_xy(&self, rope: &ropey::Rope) -> Option<XY> {
        if rope.lines().count() <= self.line as usize {
            debug!("StupidCursor.line {} > {} rope.lines().count", self.line, rope.lines().count());
            return None;
        }
        if self.line >= u16::MAX as u32 {
            debug!("StupidCursor.line {} > u16::MAX", self.line);
            return None;
        }

        let line_idx = self.line as u16;
        let line = rope.line(line_idx as usize);

        let mut x = 0 as u16;
        for g in line.to_string().graphemes(false).take(self.char_idx as usize) {
            x += g.width() as u16;
        }

        Some(XY::new(x, line_idx))
    }
}