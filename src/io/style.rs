use std::rc::Rc;

use log::debug;

use crate::primitives::colour::Colour;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Effect {
    None,
    Bold,
    Italic,
    Underline,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TextStyle {
    pub foreground: Colour,
    pub background: Colour,
    pub effect: Effect,
}

impl TextStyle {
    pub fn new(foreground: Colour, background: Colour, effect: Effect) -> Self {
        TextStyle {
            foreground,
            background,
            effect,
        }
    }

    pub fn simple(foreground: Colour, background: Colour) -> Self {
        TextStyle {
            foreground,
            background,
            effect: Effect::None,
        }
    }

    pub fn black_and_white() -> Self {
        Self::simple((255, 255, 255), (0, 0, 0))
    }
}
