use serde::{Deserialize, Serialize};

use crate::io::style::{TEXT_STYLE_WHITE_ON_BLACK, TextStyle};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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

    pub fn style(&self) -> Option<&TextStyle> {
        match self {
            Cell::Begin { style, grapheme } => {
                Some(style)
            }
            Cell::Continuation => None,
        }
    }

    pub fn grapheme(&self) -> Option<&str> {
        match self {
            Cell::Begin { style, grapheme } => {
                Some(grapheme)
            }
            Cell::Continuation => None,
        }
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
