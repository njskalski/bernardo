use std::path::Path;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use crate::config::load_error::LoadError;
use crate::config::save_error::SaveError;
use crate::io::keys::Key;
use crate::Keycode;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
    pub keyboard_config: KeyboardConfig,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct KeyboardConfig {
    #[serde(default)]
    pub global: Global,
    #[serde(default)]
    pub editor: Editor,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Global {
    pub close: Key,
    pub fuzzy_file: Key,
    pub new_buffer: Key,
}

impl Default for Global {
    fn default() -> Self {
        Global {
            close: Keycode::Char('q').to_key().with_ctrl(),
            fuzzy_file: Keycode::Char('h').to_key().with_ctrl(),
            new_buffer: Keycode::Char('n').to_key().with_ctrl(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Editor {
    pub save: Key,
    pub save_as: Key,
    pub enter_cursor_drop_mode: Key,
}

impl Default for Editor {
    fn default() -> Self {
        Editor {
            save: Keycode::Char('s').to_key().with_ctrl(),
            save_as: Keycode::Char('d').to_key().with_ctrl(),
            enter_cursor_drop_mode: Keycode::Char('e').to_key().with_ctrl(),
        }
    }
}

pub type ConfigRef = Rc<Config>;

impl Config {
    /*
    uses default filesystem (std). It is actually needed, it's unlikely that we want the theme config to be in
     */
    pub fn load_from_file(path: &Path) -> Result<Self, LoadError> {
        let b = std::fs::read(path)?;
        let s = std::str::from_utf8(&b)?;
        let item: Config = ron::from_str(s)?;
        Ok(item)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), SaveError> {
        let item_s = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new())?;
        std::fs::write(path, item_s)?;
        Ok(())
    }
}