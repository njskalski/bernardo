use std::path::Path;

use filesystem::FileSystem;
use serde::{Deserialize, Serialize};

use crate::io::style::{Effect, TextStyle};
use crate::primitives::color::Color;
use crate::primitives::cursor_set::CursorStatus;
use crate::primitives::is_default::IsDefault;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Theme {
    #[serde(default, skip_serializing_if = "GeneralCodeTheme::is_default")]
    pub general_code_theme: GeneralCodeTheme,
    #[serde(default, skip_serializing_if = "UiTheme::is_default")]
    pub ui: UiTheme,
}

impl Theme {
    pub fn name_to_theme(&self, s: &str) -> Option<Color> {
        let ct = &self;
        match s {
            "string_literal" => ct.general_code_theme.string_literal,
            "\"" => ct.general_code_theme.double_quote,
            "\'" => ct.general_code_theme.single_quote,
            "(" => ct.general_code_theme.parenthesis,
            ")" => ct.general_code_theme.parenthesis,
            "identifier" => ct.general_code_theme.identifier,
            _ => None
        }
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
    static ref HEADER_FOREGROUND : Color = ron::from_str("\"#FB4931\"").unwrap();
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
        assert_eq!(ron::from_str(r##"()"##), Ok(Theme::default()));

        assert_eq!(ron::from_str(r##"(general_code_theme:(identifier:Some("#000000")))"##),
                   Ok(Theme::default().with_general_code_theme(
                       GeneralCodeTheme::default().with_identifier(BLACK)
                   )));
    }
}