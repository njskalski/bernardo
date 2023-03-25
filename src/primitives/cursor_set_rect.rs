use log::error;

use crate::primitives::cursor::Cursor;
use crate::primitives::cursor_set::CursorSet;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::text::text_buffer::TextBuffer;

fn cursor_to_xy(c: &Cursor, buffer: &dyn TextBuffer) -> XY {
    let y = buffer.char_to_line(c.a).unwrap_or_else(|| {
        error!("failed translating cursor to XY (1), most likely wrong buffer provided. c: {:?}", c);
        0
    });

    let x = buffer.line_to_char(y)
        .map(|line_begin| c.a - line_begin)
        .unwrap_or_else(|| {
            error!("failed translating cursor to XY (2), most likely wrong buffer provided. c: {:?} y: {}", c, y);
            0
        });

    if x > u16::MAX as usize || y > u16::MAX as usize {
        error!("failed translating cursor to XY (3), x/y too big. c: {:?} x: {} y: {}", c, x, y);
        XY::ZERO
    } else {
        XY::new(x as u16, y as u16)
    }
}

pub fn cursor_set_to_rect(cs: &CursorSet, buffer: &dyn TextBuffer) -> Rect {
    if cs.set().is_empty() {
        error!("asked for cursor_rect on an empty cursor set, returning 0,0");
        return Rect::ZERO;
    }

    let first_cursor_as_xy = cursor_to_xy(&cs.set()[0], buffer);
    let mut result = Rect::new(first_cursor_as_xy, XY::ZERO);

    for i in 1..cs.set().len() {
        let cursor_as_xy = cursor_to_xy(&cs.set()[i], buffer);
        result.expand_to_contain(cursor_as_xy);
    }

    result
}