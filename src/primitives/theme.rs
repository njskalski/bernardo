// This is a basic structure holding drawing parameters for a widget.
// It's not very well thought.

use crate::io::style::{Effect, TextStyle};
use crate::primitives::color::Color;
use crate::primitives::colors;

pub struct Theme {
    active_background: Color,
    inactive_background: Color,
    active_input_background: Color,
    inactive_input_background: Color,
    active_input_foreground: Color,
    inactive_input_foreground: Color,

    active_cursor_background: Color,
    inactive_cursor_background: Color,
}

impl Theme {
    pub const fn default() -> Self {
        Theme {
            active_background: colors::COLOR_BRIGHT_BACKGROUND,
            inactive_background: colors::COLOR_DARK_BACKGROUND,
            active_input_background: colors::COLOR_ACTIVE_INPUT_BACKGROUND,
            inactive_input_background: colors::COLOR_INACTIVE_INPUT_BACKGROUND,
            active_input_foreground: colors::COLOR_PINK_SAND,
            inactive_input_foreground: colors::COLOR_INACTIVE_INPUT_FOREGROUND,
            active_cursor_background: colors::COLOR_SKY,
            inactive_cursor_background: colors::COLOR_INACTIVE_INPUT_BACKGROUND,
        }
    }

    pub fn active_edit(&self) -> TextStyle {
        TextStyle::new(self.active_input_foreground, self.active_input_background, Effect::None)
    }

    pub fn inactive_edit(&self) -> TextStyle {
        TextStyle::new(self.inactive_input_foreground, self.inactive_input_background, Effect::None)
    }

    pub fn active_cursor(&self) -> TextStyle {
        TextStyle::new(self.active_input_foreground, self.active_cursor_background, Effect::None)
    }

    pub fn header(&self) -> TextStyle {
        TextStyle::new(self.inactive_input_foreground, self.inactive_input_background, Effect::Underline)
    }
}