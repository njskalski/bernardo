use std::fs;
use std::path::{Path, PathBuf};

use crate::config::load_error::LoadError;
use crate::config::save_error::SaveError;
use crate::cursor::cursor::CursorStatus;
use crate::gladius::load_config::get_config_dir;
use crate::io::style::{Effect, TextStyle};
use crate::primitives::color::Color;
use crate::primitives::tmtheme::TmTheme;
use lazy_static::lazy_static;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use syntect::highlighting::ThemeSet;
use syntect::LoadingError;

// TODO get rid of clone (in mock output we need Rc/Arc)
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Theme {
    #[serde(default)]
    pub ui: UiTheme,
    // I do not serialize this, use the default value and always say "true" in comparison operator.
    #[serde(default, skip)]
    pub tm: TmTheme,

    pub tm_theme_name: Option<String>,
}

impl Theme {
    pub fn name_to_color(&self, s: &str) -> Option<Color> {
        if let Some(color) = self.tm.name_to_color(s) {
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
            CursorStatus::UnderCursor => Some(self.ui.cursors.secondary_anchor_background),
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

    pub fn editor_label_warning(&self) -> TextStyle {
        self.ui.label_warning
    }

    pub fn editor_label_error(&self) -> TextStyle {
        self.ui.label_error
    }

    pub fn editor_label_type_annotation(&self) -> TextStyle {
        self.ui.label_type_annotation
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UiTheme {
    pub non_focused: TextStyle,
    pub focused: TextStyle,
    pub focused_highlighted: TextStyle,
    pub non_focused_highlighted: TextStyle,
    pub header: TextStyle,
    pub cursors: CursorsSettings,

    // not sure if I should not rearrange this
    pub mode_2_background: Color,
    pub label_warning: TextStyle,
    pub label_error: TextStyle,
    pub label_type_annotation: TextStyle,
}

lazy_static! {

    // Some tests will start failing if default (focused, unfocused) x (higlight, non-highlight) matrix has non-unique cells.
    // (like one in nested menu interpreter, that relies on these colors to tell apart what's what)
    static ref DEFAULT_FOCUSED_BACKGROUND: Color = ron::from_str("\"#282828\"").unwrap();
    static ref DEFAULT_FOCUSED_FOREGROUND: Color = ron::from_str("\"#928374\"").unwrap();
    static ref DEFAULT_NON_FOCUSED_BACKGROUND: Color = ron::from_str("\"#181818\"").unwrap();
    static ref DEFAULT_NON_FOCUSED_FOREGROUND: Color = ron::from_str("\"#928374\"").unwrap();
    static ref HIGHLIGHTED_FOCUSED_BACKGROUND: Color = ron::from_str("\"#383433\"").unwrap();
    static ref HIGHLIGHTED_FOCUSED_FOREGROUND: Color = ron::from_str("\"#928384\"").unwrap();
    static ref HIGHLIGHTED_NON_FOCUSED_BACKGROUND: Color = ron::from_str("\"#181818\"").unwrap();
    static ref HIGHLIGHTED_NON_FOCUSED_FOREGROUND: Color = ron::from_str("\"#928384\"").unwrap();
    static ref PRIMARY_CURSOR_ANCHOR_BACKGROUND: Color = ron::from_str("\"#FFB81C\"").unwrap();
    static ref SECONDARY_CURSORS_ANCHOR_BACKGROUND: Color = ron::from_str("\"#ED7737\"").unwrap();
    static ref CURSORS_BACKGROUND: Color = ron::from_str("\"#852F00\"").unwrap();
    static ref CURSORS_FOREGROUND: Color = ron::from_str("\"#FFC4A3\"").unwrap();
    static ref HEADER_BACKGROUND: Color = *HIGHLIGHTED_FOCUSED_BACKGROUND;
    static ref HEADER_FOREGROUND: Color = ron::from_str("\"#AC5894\"").unwrap();
    static ref MODE2_BACKGROUND: Color = ron::from_str("\"#122322\"").unwrap();
    static ref MUSTARD_COLOR: Color = ron::from_str("\"#EABE38\"").unwrap();
    static ref KETCHUP_COLOR: Color = ron::from_str("\"#B10B0B\"").unwrap();
    static ref BLACK_COLOR: Color = ron::from_str("\"#000000\"").unwrap();
    static ref GREY_COLOR: Color = ron::from_str("\"#999999\"").unwrap();
}

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
            label_warning: TextStyle {
                foreground: *BLACK_COLOR,
                background: *MUSTARD_COLOR,
                effect: Default::default(),
            },
            label_error: TextStyle {
                foreground: *BLACK_COLOR,
                background: *KETCHUP_COLOR,
                effect: Default::default(),
            },
            label_type_annotation: TextStyle {
                foreground: *BLACK_COLOR,
                background: *GREY_COLOR,
                effect: Default::default(),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CursorsSettings {
    pub primary_anchor_background: Color,
    pub secondary_anchor_background: Color,
    pub background: Color,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foreground: Option<Color>,
}

const DEFAULT_THEME_PATH: &str = "themes/default.ron";

impl Theme {
    const DEFAULT_TM_THEME_FOLDER: &'static str = "tm_themes";

    pub fn get_theme_dir() -> PathBuf {
        let config_dir = get_config_dir();
        config_dir.join(Self::DEFAULT_TM_THEME_FOLDER)
    }

    /*
    uses default filesystem (std). It is actually needed, the config might need to be initialized before filesystem AND it's most likely
    not local to any workspace.
     */
    pub fn load_from_file(path: &Path) -> Result<Self, LoadError> {
        let b = std::fs::read(path)?;
        let s = std::str::from_utf8(&b)?;
        let mut item: Theme = ron::from_str(s)?;

        if let Some(name) = item.tm_theme_name.as_ref() {
            let mut theme_set = ThemeSet::load_defaults();
            let themes_dir = Self::get_theme_dir();
            if themes_dir.exists() {
                match theme_set.add_from_folder(&themes_dir) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("failed loading themes from {}: {}", Self::DEFAULT_TM_THEME_FOLDER, e);
                    }
                }
            } else {
                warn!("tm themes dir [{:?}] does not exist", &themes_dir);
            }

            if let Some(theme) = theme_set.themes.get(name) {
                item.tm = TmTheme::new(theme.clone());
            } else {
                error!("did not find theme {}. Available themes: {:?}", name, theme_set.themes.keys());
            }
        }

        Ok(item)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), SaveError> {
        let item_s = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new())?;
        path.parent().map(|p| {
            fs::create_dir_all(p).map_err(|e| {
                error!("failed creating dir {:?}: {}", p, e);
            })
        });

        fs::write(path, item_s)?;

        Ok(())
    }

    pub fn load_or_create_default(root_config_dir: &Path) -> Result<Self, LoadError> {
        let full_path = root_config_dir.join(DEFAULT_THEME_PATH);
        if full_path.exists() {
            Self::load_from_file(&full_path)
        } else {
            let theme = Self::default();
            theme
                .save_to_file(&full_path)
                .map_err(|e| {
                    error!("failed saving theme to {:?}: {}", &full_path, e);
                })
                .unwrap();
            Ok(theme)
        }
    }
}
