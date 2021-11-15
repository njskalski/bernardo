// This is a basic structure holding drawing parameters for a widget.
// It's not very well thought.

use crate::io::style::{Effect, TextStyle};
use crate::primitives::color::Color;
use crate::primitives::colors;
use crate::primitives::colors::{COLOR_CURSOR_BACKGROUND, COLOR_CURSOR_FOREGROUND};

pub struct Theme {
    cursor_background: Color,
    cursor_foreground: Color,
    default_text_bg: Color,
    default_text_fg: Color,
    edit_background: Color,
    edit_foreground: Color,
    header_background: Color,
    header_foreground: Color,
}

impl Theme {
    pub const fn default() -> Self {
        Theme {
            cursor_background: colors::COLOR_CURSOR_BACKGROUND,
            cursor_foreground: colors::COLOR_CURSOR_FOREGROUND,
            default_text_bg: colors::DEFAULT_TEXT_BACKGROUND,
            default_text_fg: colors::DEFAULT_TEXT_FOREGROUND,
            edit_background: colors::EDIT_BACKGROUND,
            edit_foreground: colors::EDIT_FOREGROUND,
            header_background: colors::HEADER_BACKGROUND,
            header_foreground: colors::HEADER_FOREGROUND,
        }
    }

    pub fn cursor(&self) -> TextStyle {
        TextStyle::new(self.cursor_foreground, self.cursor_background, Effect::None)
    }

    pub fn header(&self) -> TextStyle {
        TextStyle::new(self.header_foreground, self.header_background, Effect::Underline)
    }

    pub fn default_text(&self) -> TextStyle {
        TextStyle::new(self.default_text_fg, self.default_text_bg, Effect::None)
    }

    pub fn editable_field(&self) -> TextStyle {
        TextStyle::new(self.edit_foreground, self.edit_background, Effect::None)
    }
}