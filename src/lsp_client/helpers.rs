use log::error;

use crate::primitives::cursor_set::Cursor;
use crate::primitives::xy::XY;
use crate::text::buffer::Buffer;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LspTextCursor {
    pub col: u32,
    pub row: u32,
}

impl LspTextCursor {
    pub fn to_xy(&self) -> Option<XY> {
        if self.col < u16::MAX as u32 && self.row < u16::MAX as u32 {
            Some(XY::new(self.col as u16, self.row as u16))
        } else {
            None
        }
    }
}

pub fn get_lsp_text_cursor(buffer: &dyn Buffer, cursor: &Cursor) -> Result<LspTextCursor, ()> {
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

    let col = (cursor.a - begin_line);

    Ok(LspTextCursor {
        row: line as u32,
        col: col as u32,
    })
}