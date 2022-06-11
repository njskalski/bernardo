use std::path::{Path, PathBuf};
use std::rc::Rc;
use serde::{Deserialize, Serialize};

use crate::config::load_error::LoadError;
use crate::config::save_error::SaveError;
use crate::io::keys::Key;
use crate::Keycode;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
    pub keyboard_config: KeyboardConfig,

    #[serde(skip)]
    pub config_dir: PathBuf,
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
    pub browse_buffers: Key,
    pub everything_bar: Key,
}

impl Default for Global {
    fn default() -> Self {
        Global {
            close: Keycode::Char('q').to_key().with_ctrl(),
            fuzzy_file: Keycode::Char('h').to_key().with_ctrl(),
            new_buffer: Keycode::Char('n').to_key().with_ctrl(),
            browse_buffers: Keycode::Char('w').to_key().with_ctrl(),
            // This is the most important feature of them all.
            // In order to support it EVERYWHERE it will need to be converted to InputEvent
            everything_bar: Keycode::Char('e').to_key().with_ctrl(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Editor {
    pub save: Key,
    pub save_as: Key,
    pub enter_cursor_drop_mode: Key,

    pub find: Key,
    pub replace: Key,
    pub close_find_replace: Key,
}

impl Default for Editor {
    fn default() -> Self {
        Editor {
            save: Keycode::Char('s').to_key().with_ctrl(),
            save_as: Keycode::Char('d').to_key().with_ctrl(),
            enter_cursor_drop_mode: Keycode::Char('e').to_key().with_ctrl(),
            find: Keycode::Char('f').to_key().with_ctrl(),
            replace: Keycode::Char('r').to_key().with_ctrl(),
            close_find_replace: Keycode::Esc.to_key(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_ser_de() {
        let d = Global::default();
        let item = ron::ser::to_string_pretty(&d, ron::ser::PrettyConfig::new());
        assert_eq!(item.as_ref().err(), None);
        let read = ron::from_str::<Global>(item.as_ref().unwrap());
        assert_eq!(read.as_ref().err(), None);
    }

    #[test]
    fn test_editor_ser_de() {
        let d = Editor::default();
        let item = ron::ser::to_string_pretty(&d, ron::ser::PrettyConfig::new());
        assert_eq!(item.as_ref().err(), None);
        let read = ron::from_str::<Editor>(item.as_ref().unwrap());
        assert_eq!(read.as_ref().err(), None);
    }

    #[test]
    fn test_config_ser_de() {
        let d = Config::default();
        let item = ron::ser::to_string_pretty(&d, ron::ser::PrettyConfig::new());
        assert_eq!(item.as_ref().err(), None);
        let read = ron::from_str::<Config>(item.as_ref().unwrap());
        assert_eq!(read.as_ref().err(), None);
    }
}