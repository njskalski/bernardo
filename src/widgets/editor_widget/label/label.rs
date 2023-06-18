use std::ops::Range;

use crate::cursor::cursor::Cursor;
use crate::io::style::TextStyle;
use crate::primitives::printable::Printable;
use crate::text::text_buffer::TextBuffer;

pub enum LabelPos {
    /*
    Appears immediately after anchoring symbol, can be cursor selected for context
     */
    Inline { cursor: Cursor },
    /*
        both line and column are 1-based
     */
    InlineStupid { line_no_1b: usize, col_no_1b: usize },

    /*
    Appears above indexed line (1-based)
     */
    LineAbove { line_no_1b: usize },
    /*
    Appears below indexed line (1-based)
     */
    LineBelow { line_no_1b: usize },

    /*
    Appears after indexed line (1-based)
     */
    LineAfter { line_no_1b: usize },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum LabelStyle {
    Warning,
    Error,
    Random(TextStyle),
}

pub struct Label {
    // TODO make private
    pub pos: LabelPos,
    pub style: LabelStyle,
    pub contents: Box<dyn Printable>,
}

