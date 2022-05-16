use serde::{Deserialize, Serialize};
use crate::io::keys::Key;
use crate::Keycode;


#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
    // #[serde(default, skip_serializing_if = "UiTheme::is_default")]
    // pub ui: UiTheme,
    // // I do not serialize this, use the default value and always say "true" in comparison operator.
    // #[serde(default, skip_serializing)]
    // pub tm: TmTheme,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct KeyboardConfig {
    #[serde(default)]
    global: Global,
    #[serde(default)]
    editor: Editor,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Global {
    close: Key,
    fuzzy_file: Key,
}

impl Default for Global {
    fn default() -> Self {
        Global {
            close: Keycode::Char('q').to_key().with_ctrl(),
            fuzzy_file: Keycode::Char('h').to_key().with_ctrl(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Editor {
    enter_cursor_drop_mode: Key,
}

impl Default for Editor {
    fn default() -> Self {
        Editor {
            enter_cursor_drop_mode: Keycode::Char('e').to_key().with_ctrl(),
        }
    }
}