use crate::io::style::{TextStyle, TextStyle_WhiteOnBlack};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Cell {
    Begin { style: TextStyle, grapheme: String },
    Continuation,
}

impl Cell {
    pub fn empty() -> Cell {
        Cell::Begin {
            style: TextStyle_WhiteOnBlack,
            grapheme: ' '.into(),
        }
    }

    pub fn new(style: TextStyle, grapheme: String) -> Cell {
        Cell::Begin { style, grapheme }
    }

    pub fn continuation() -> Cell {
        Cell::Continuation
    }
}
