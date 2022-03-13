use crate::io::style::{TEXT_STYLE_WHITE_ON_BLACK, TextStyle};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Cell {
    Begin { style: TextStyle, grapheme: String },
    Continuation,
}

impl Cell {
    pub fn new(style: TextStyle, grapheme: String) -> Cell {
        Cell::Begin { style, grapheme }
    }

    pub fn continuation() -> Cell {
        Cell::Continuation
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Begin {
            style: TEXT_STYLE_WHITE_ON_BLACK,
            grapheme: ' '.into(),
        }
    }
}
