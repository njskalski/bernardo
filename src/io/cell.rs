use crate::io::style::TextStyle;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Cell {
    Begin { style: TextStyle, grapheme: String },
    Continuation,
}

impl Cell {
    pub fn empty() -> Cell {
        Cell::Begin {
            style: TextStyle::black_and_white(),
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
