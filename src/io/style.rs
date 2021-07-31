use std::rc::Rc;

use log::debug;

use crate::primitives::color::Color;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Effect {
    None,
    Bold,
    Italic,
    Underline,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TextStyle {
    pub foreground: Color,
    pub background: Color,
    pub effect: Effect,
}

impl TextStyle {
    pub fn new(foreground: Color, background: Color, effect: Effect) -> Self {
        TextStyle {
            foreground,
            background,
            effect,
        }
    }

    pub fn simple(foreground: Color, background: Color) -> Self {
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
