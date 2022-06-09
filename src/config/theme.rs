use std::path::Path;
use log::warn;
use serde::{Deserialize, Serialize};
use crate::config::load_error::LoadError;
use crate::config::save_error::SaveError;
use crate::fs::file_front::FileFront;

use crate::io::style::{Effect, TextStyle};
use crate::primitives::color::Color;
use crate::primitives::cursor_set::CursorStatus;
use crate::primitives::is_default::IsDefault;
use crate::primitives::tmtheme::TmTheme;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Theme {
    #[serde(default, skip_serializing_if = "UiTheme::is_default")]
    pub ui: UiTheme,
    // I do not serialize this, use the default value and always say "true" in comparison operator.
    #[serde(default, skip_serializing)]
    pub tm: TmTheme,
}

impl Theme {
    pub fn name_to_theme(&self, s: &str) -> Option<Color> {
        if let Some(color) = self.tm.color_for_name(s) {
            return Some(color);
        }


        warn!("not matched code identifier \"{}\"", s);
        None
    }

    pub fn default_text(&self, focused: bool) -> TextStyle {
        if focused {
            self.ui.focused
        } else {
            self.ui.non_focused
        }
    }

    pub fn highlighted(&self, focused: bool) -> TextStyle {
        if focused {
            self.ui.focused_highlighted
        } else {
            self.ui.non_focused_highlighted
        }
    }

    pub fn cursor_background(&self, cs: CursorStatus) -> Option<Color> {
        match cs {
            CursorStatus::None => None,
            CursorStatus::WithinSelection => Some(self.ui.cursors.background),
            CursorStatus::UnderCursor => Some(self.ui.cursors.secondary_anchor_background)
        }
    }

    pub fn cursor_foreground(&self) -> Option<Color> {
        self.ui.cursors.foreground
    }

    pub fn special_cursor_background(&self) -> Color {
        self.ui.cursors.primary_anchor_background
    }

    pub fn header(&self, focused: bool) -> TextStyle {
        if focused {
            self.ui.header
        } else {
            self.ui.header.with_background(self.ui.non_focused.background)
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UiTheme {
    pub non_focused: TextStyle,
    pub focused: TextStyle,
    pub focused_highlighted: TextStyle,
    pub non_focused_highlighted: TextStyle,
    pub header: TextStyle,
    pub cursors: CursorsSettings,
    // not sure if I should not rearrange this
    pub mode_2_background: Color,
}

lazy_static!(
    static ref DEFAULT_FOCUSED_BACKGROUND : Color = ron::from_str("\"#282828\"").unwrap();
    static ref DEFAULT_FOCUSED_FOREGROUND : Color = ron::from_str("\"#928374\"").unwrap();

    static ref DEFAULT_NON_FOCUSED_BACKGROUND : Color = ron::from_str("\"#181818\"").unwrap();
    static ref DEFAULT_NON_FOCUSED_FOREGROUND : Color = ron::from_str("\"#928374\"").unwrap();

    static ref HIGHLIGHTED_FOCUSED_BACKGROUND : Color = ron::from_str("\"#383433\"").unwrap();
    static ref HIGHLIGHTED_FOCUSED_FOREGROUND : Color = ron::from_str("\"#928374\"").unwrap();

    static ref HIGHLIGHTED_NON_FOCUSED_BACKGROUND : Color = ron::from_str("\"#181818\"").unwrap();
    static ref HIGHLIGHTED_NON_FOCUSED_FOREGROUND : Color = ron::from_str("\"#928374\"").unwrap();

    static ref PRIMARY_CURSOR_ANCHOR_BACKGROUND : Color = ron::from_str("\"#FFB81C\"").unwrap();
    static ref SECONDARY_CURSORS_ANCHOR_BACKGROUND : Color = ron::from_str("\"#ED7737\"").unwrap();
    static ref CURSORS_BACKGROUND : Color = ron::from_str("\"#852F00\"").unwrap();
    static ref CURSORS_FOREGROUND : Color = ron::from_str("\"#FFC4A3\"").unwrap();

    static ref HEADER_BACKGROUND : Color = *HIGHLIGHTED_FOCUSED_BACKGROUND;
    static ref HEADER_FOREGROUND : Color = ron::from_str("\"#AC5894\"").unwrap();

    static ref MODE2_BACKGROUND : Color = ron::from_str("\"#122322\"").unwrap();
);

impl Default for UiTheme {
    fn default() -> Self {
        UiTheme {
            non_focused: TextStyle {
                foreground: *DEFAULT_NON_FOCUSED_FOREGROUND,
                background: *DEFAULT_NON_FOCUSED_BACKGROUND,
                effect: Effect::None,
            },
            focused: TextStyle {
                foreground: *DEFAULT_FOCUSED_FOREGROUND,
                background: *DEFAULT_FOCUSED_BACKGROUND,
                effect: Effect::None,
            },
            focused_highlighted: TextStyle {
                foreground: *HIGHLIGHTED_FOCUSED_FOREGROUND,
                background: *HIGHLIGHTED_FOCUSED_BACKGROUND,
                effect: Effect::None,
            },
            non_focused_highlighted: TextStyle {
                foreground: *HIGHLIGHTED_NON_FOCUSED_FOREGROUND,
                background: *HIGHLIGHTED_NON_FOCUSED_BACKGROUND,
                effect: Effect::None,
            },
            header: TextStyle {
                foreground: *HEADER_FOREGROUND,
                background: *HEADER_BACKGROUND,
                effect: Effect::None,
            },
            cursors: CursorsSettings {
                primary_anchor_background: *PRIMARY_CURSOR_ANCHOR_BACKGROUND,
                secondary_anchor_background: *SECONDARY_CURSORS_ANCHOR_BACKGROUND,
                background: *CURSORS_BACKGROUND,
                foreground: Some(*CURSORS_FOREGROUND),
            },
            mode_2_background: *MODE2_BACKGROUND,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CursorsSettings {
    pub primary_anchor_background: Color,
    pub secondary_anchor_background: Color,
    pub background: Color,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foreground: Option<Color>,
}

impl Theme {
    /*
    uses default filesystem (std). It is actually needed, it's unlikely that we want the theme config to be in
     */
    pub fn load_from_file(path: &Path) -> Result<Self, LoadError> {
        let b = std::fs::read(path)?;
        let s = std::str::from_utf8(&b)?;
        let item = ron::from_str(s)?;
        Ok(item)
    }

    pub fn load_from_ff(ff: &FileFront) -> Result<Self, LoadError> {
        let s = ff.read_entire_file_to_rope()?.to_string();
        Ok(ron::from_str(s.as_str())?)
    }

    pub fn save_to_ff(&self, ff: &FileFront) -> Result<(), SaveError> {
        let item_s = ron::to_string(self)?;
        ff.overwrite_with(&item_s.as_str())?;
        Ok(())
    }
}
