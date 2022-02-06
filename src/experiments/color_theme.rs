use std::collections::HashMap;
use std::fs;
use std::path::Path;

use filesystem::FileSystem;
use json;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::primitives::color::Color;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ColorTheme {
    #[serde(default, skip_serializing_if = "GeneralCodeTheme::is_default")]
    pub general_code_theme: GeneralCodeTheme,
}

impl ColorTheme {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }

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