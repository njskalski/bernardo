// This is a basic structure holding drawing parameters for a widget.
// It's not very well thought.

use crate::experiments::focus_group::FocusGroup;
use crate::io::style::{Effect, TextStyle};
use crate::primitives::color::Color;
use crate::primitives::colors;

pub struct Theme {
    cursor_background: Color,
    cursor_foreground: Color,
    default_text_bg: Color,
    default_text_fg: Color,
    edit_background: Color,
    edit_foreground: Color,
    header_background: Color,
    header_foreground: Color,

    non_focused_background: Color,
    focused_background: Color,
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
            non_focused_background: colors::NON_FOCUSED_FOREGROUND,
            focused_background: colors::FOCUSED_BACKGROUND,
        }
    }

    pub fn cursor(&self) -> TextStyle {
        TextStyle::new(self.cursor_foreground, self.cursor_background, Effect::None)
    }

    pub fn header(&self) -> TextStyle {
        TextStyle::new(self.header_foreground, self.header_background, Effect::Underline)
    }

    pub fn default_text(&self, focused: bool) -> TextStyle {
        TextStyle::new(self.default_text_fg,
                       self.default_background(focused),
                       Effect::None)
    }

    pub fn editable_field(&self) -> TextStyle {
        TextStyle::new(self.edit_foreground, self.edit_background, Effect::None)
    }

    pub fn default_background(&self, focused: bool) -> Color {
        if focused { self.focused_background } else { self.non_focused_background }
    }
}