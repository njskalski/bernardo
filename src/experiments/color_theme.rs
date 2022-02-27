use std::collections::HashMap;
use std::fs;
use std::path::Path;

use filesystem::FileSystem;
use json;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::io::style::{Effect, TextStyle};
use crate::primitives::color::{Color, WHITE};
use crate::primitives::colors::{COLOR_CURSOR_BACKGROUND, DEFAULT_TEXT_FOREGROUND, FOCUSED_BACKGROUND};
use crate::primitives::is_default::IsDefault;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ColorTheme {
    #[serde(default, skip_serializing_if = "GeneralCodeTheme::is_default")]
    pub general_code_theme: GeneralCodeTheme,
    #[serde(default, skip_serializing_if = "UiTheme::is_default")]
    pub ui: UiTheme,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UiTheme {
    pub non_focused: TextStyle,
    pub focused: TextStyle,
    pub highligted: TextStyle,
    pub cursors: CursorsSettings,
}

lazy_static!(
    static ref DEFAULT_NON_FOCUSED_BACKGROUND : Color = ron::from_str("#181818").unwrap();
    static ref DEFAULT_NON_FOCUSED_FOREGROUND : Color = ron::from_str("#928374").unwrap();

    static ref DEFAULT_FOCUSED_BACKGROUND : Color = ron::from_str("#282828").unwrap();
    static ref DEFAULT_FOCUSED_FOREGROUND : Color = ron::from_str("#928374").unwrap();

    static ref DEFAULT_HIGHLIGHTED_BACKGROUND : Color = ron::from_str("#383433").unwrap();
    static ref DEFAULT_HIGHLIGHTED_FOREGROUND : Color = ron::from_str("#928374").unwrap();

    static ref PRIMARY_CURSOR_ANCHOR_BACKGROUND : Color = ron::from_str("#FFB81C").unwrap();
    static ref SECONDARY_CURSORS_ANCHOR_BACKGROUND : Color = ron::from_str("#ED7737").unwrap();
    static ref CURSORS_BACKGROUND : Color = ron::from_str("#852F00").unwrap();
    static ref CURSORS_FOREGROUND : Color = ron::from_str("#FFC4A3").unwrap();
);

impl Default for UiTheme {
    fn default() -> Self {
        UiTheme {
            non_focused: TextStyle {
                foreground: *DEFAULT_NON_FOCUSED_BACKGROUND,
                background: *DEFAULT_NON_FOCUSED_FOREGROUND,
                effect: Effect::None,
            },
            focused: TextStyle {
                foreground: *DEFAULT_FOCUSED_BACKGROUND,
                background: *DEFAULT_FOCUSED_FOREGROUND,
                effect: Effect::None,
            },
            highligted: TextStyle {
                foreground: *DEFAULT_HIGHLIGHTED_BACKGROUND,
                background: *DEFAULT_HIGHLIGHTED_FOREGROUND,
                effect: Effect::None,
            },
            cursors: CursorsSettings {
                primary_anchor_background: *PRIMARY_CURSOR_ANCHOR_BACKGROUND,
                secondary_anchor_background: *SECONDARY_CURSORS_ANCHOR_BACKGROUND,
                background: *CURSORS_BACKGROUND,
                foreground: Some(*CURSORS_FOREGROUND),
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CursorsSettings {
    pub primary_anchor_background: Color,
    pub secondary_anchor_background: Color,
    pub background: Color,
    #[serde(default, skip_serializing_if = "Option::is_default")]
    pub foreground: Option<Color>,
}

impl ColorTheme {
    pub fn with_general_code_theme(self, general_code_theme: GeneralCodeTheme) -> Self {
        Self {
            general_code_theme,
            ..self
        }
    }

    pub fn load_from_file<T: FileSystem>(fs: T, path: &Path) -> Result<Self, ron::Error> {
        let data = fs.read_file_to_string(path)?;
        ron::from_str(&data)
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct GeneralCodeTheme {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub string_literal: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_identifier: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parenthesis: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bracket: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub double_quote: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub single_quote: Option<Color>,
}

impl GeneralCodeTheme {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }

    pub fn with_identifier(self, c: Color) -> Self {
        Self {
            identifier: Some(c),
            ..self
        }
    }

    pub fn with_string_literal(self, c: Color) -> Self {
        Self {
            string_literal: Some(c),
            ..self
        }
    }

    pub fn with_type_identifier(self, c: Color) -> Self {
        Self {
            type_identifier: Some(c),
            ..self
        }
    }

    pub fn with_parenthesis(self, c: Color) -> Self {
        Self {
            parenthesis: Some(c),
            ..self
        }
    }

    pub fn with_bracket(self, c: Color) -> Self {
        Self {
            bracket: Some(c),
            ..self
        }
    }

    pub fn with_double_quote(self, c: Color) -> Self {
        Self {
            double_quote: Some(c),
            ..self
        }
    }

    pub fn with_single_quote(self, c: Color) -> Self {
        Self {
            single_quote: Some(c),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::color::{BLACK, WHITE};

    use super::*;

    #[test]
    fn test_serialize_general() {
        assert_eq!(ron::to_string(&GeneralCodeTheme::default()), Ok("()".to_string()));

        assert_eq!(ron::to_string(
            &GeneralCodeTheme::default()
                .with_identifier(BLACK)),
                   Ok(r##"(identifier:Some("#000000"))"##.to_string()));

        assert_eq!(ron::to_string(
            &GeneralCodeTheme::default()
                .with_string_literal(WHITE)
                .with_bracket(WHITE)),
                   Ok(r##"(string_literal:Some("#FFFFFF"),bracket:Some("#FFFFFF"))"##.to_string()));
    }

    #[test]
    fn test_deserialize_theme() {
        assert_eq!(ron::from_str(r##"()"##), Ok(ColorTheme::default()));

        assert_eq!(ron::from_str(r##"(general_code_theme:(identifier:Some("#000000")))"##),
                   Ok(ColorTheme::default().with_general_code_theme(
                       GeneralCodeTheme::default().with_identifier(BLACK)
                   )));
    }
}