use std::ops::Range;
use crate::cursor::cursor::Cursor;
use crate::primitives::printable::Printable;
use crate::text::text_buffer::TextBuffer;

pub enum LabelPos {
    /*
    Appears immediately after anchoring symbol, can be cursor selected for context
     */
    Inline{cursor : Cursor},
    /*
    Appears above indexed line
     */
    InlineAbove{ idx : usize },
    InlineBelow{ idx : usize },
}

pub struct Label {
    pos : LabelPos,
    contents : Box<dyn Printable>
}

pub trait LabelProvider {
    fn query_for(&self, buffer: &dyn TextBuffer, char_range : Range<usize>) -> Box<dyn Iterator<Item=Label>>;
}
