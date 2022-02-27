use serde::{Deserialize, Serialize};

use crate::primitives::color::Color;
use crate::primitives::is_default::IsDefault;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum Effect {
    None,
    Bold,
    Italic,
    Underline,
}

impl Default for Effect {
    fn default() -> Self {
        Effect::None
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub struct TextStyle {
    pub foreground: Color,
    pub background: Color,
    #[serde(default, skip_serializing_if = "Effect::is_default")]
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

    pub fn half(&self) -> Self {
        TextStyle {
            foreground: self.foreground.half(),
            background: self.background.half(),
            effect: self.effect,
        }
    }

    pub fn maybe_half(&self, focused: bool) -> Self {
        if focused {
            *self
        } else {
            self.half()
        }
    }

    pub fn with_foreground(self, foreground: Color) -> Self {
        Self {
            foreground,
            ..self
        }
    }

    pub fn with_background(self, background: Color) -> Self {
        Self {
            background,
            ..self
        }
    }

    pub fn with_effect(self, effect: Effect) -> Self {
        Self {
            effect,
            ..self
        }
    }
}

pub const TextStyle_WhiteOnBlack: TextStyle = TextStyle {
    foreground: Color { R: 255, G: 255, B: 255 },
    background: Color { R: 0, G: 0, B: 0 },
    effect: Effect::None,
};

pub const TextStyle_WhiteOnBlue: TextStyle = TextStyle {
    foreground: Color { R: 255, G: 255, B: 255 },
    background: Color { R: 100, G: 102, B: 237 },
    effect: Effect::None,
};

pub const TextStyle_WhiteOnYellow: TextStyle = TextStyle {
    foreground: Color { R: 255, G: 255, B: 255 },
    background: Color { R: 237, G: 207, B: 126 },
    effect: Effect::None,
};

pub const TextStyle_WhiteOnBrightYellow: TextStyle = TextStyle {
    foreground: Color { R: 255, G: 255, B: 255 },
    background: Color { R: 237, G: 226, B: 164 },
    effect: Effect::None,
};

pub const TextStyle_WhiteOnRedish: TextStyle = TextStyle {
    foreground: Color { R: 255, G: 255, B: 255 },
    background: Color { R: 201, G: 81, B: 73 },
    effect: Effect::None,
};
