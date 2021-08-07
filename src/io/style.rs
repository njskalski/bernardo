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
}

pub const TextStyle_WhiteOnBlack: TextStyle = TextStyle {
    foreground: (255, 255, 255),
    background: (0, 0, 0),
    effect: Effect::None
};

pub const TextStyle_WhiteOnBlue: TextStyle = TextStyle {
    foreground: (255, 255, 255),
    background: (100, 102, 237),
    effect: Effect::None
};

pub const TextStyle_WhiteOnYellow: TextStyle = TextStyle {
    foreground: (255, 255, 255),
    background: (237, 207, 126),
    effect: Effect::None
};

pub const TextStyle_WhiteOnBrightYellow: TextStyle = TextStyle {
    foreground: (255, 255, 255),
    background: (237, 226, 164),
    effect: Effect::None
};

pub const TextStyle_WhiteOnRedish: TextStyle = TextStyle {
    foreground: (255, 255, 255),
    background: (201, 81, 73),
    effect: Effect::None
};
