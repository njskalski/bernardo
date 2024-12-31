use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::config::global_editor_options::GlobalEditorOptions;
use crate::config::load_error::LoadError;
use crate::config::save_error::SaveError;
use crate::io::keys::{Key, Keycode};
use crate::primitives::is_default::IsDefault;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
    pub keyboard_config: KeyboardConfig,

    #[serde(skip)]
    pub config_dir: PathBuf,

    #[serde(default, skip_serializing_if = "GlobalEditorOptions::is_default")]
    pub global: GlobalEditorOptions,

    pub learning_mode: bool,

    #[serde(default)]
    pub file_tree_view_options: FileTreeViewOptions,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FileTreeViewOptions {
    pub show_hidden_files: bool,
}

impl Default for FileTreeViewOptions {
    fn default() -> Self {
        FileTreeViewOptions { show_hidden_files: false }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct KeyboardConfig {
    #[serde(default)]
    pub global: Global,
    #[serde(default)]
    pub editor: Editor,

    #[serde(default)]
    pub file_tree: FileTree,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Global {
    pub browse_buffers: Key,
    pub close: Key,
    pub close_context_menu: Key,
    pub everything_bar: Key,
    pub find_in_files: Key,
    pub fuzzy_file: Key,
    pub new_buffer: Key,
    pub make_screenshot: Key,
    pub next_display: Key,
    pub prev_display: Key,
}

impl Default for Global {
    fn default() -> Self {
        Global {
            close: Keycode::Char('q').to_key().with_ctrl(),
            fuzzy_file: Keycode::Char('j').to_key().with_ctrl(),
            new_buffer: Keycode::Char('n').to_key().with_ctrl(),
            browse_buffers: Keycode::Char('b').to_key().with_ctrl(),
            // This is the most important feature of them all.
            // In order to support it EVERYWHERE it will need to be converted to InputEvent
            everything_bar: Keycode::Char('e').to_key().with_ctrl(),
            find_in_files: Keycode::Char('g').to_key().with_ctrl(),
            close_context_menu: Keycode::Esc.to_key(),
            make_screenshot: Keycode::Char('u').to_key().with_ctrl(),
            next_display: Keycode::Char('.').to_key().with_alt(),
            prev_display: Keycode::Char(',').to_key().with_alt(),
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
    pub request_completions: Key,

    pub reformat: Key,
}

impl Default for Editor {
    fn default() -> Self {
        Editor {
            save: Keycode::Char('s').to_key().with_ctrl(),
            save_as: Keycode::Char('d').to_key().with_ctrl(),
            enter_cursor_drop_mode: Keycode::Char('w').to_key().with_ctrl(),
            find: Keycode::Char('f').to_key().with_ctrl(),
            replace: Keycode::Char('r').to_key().with_ctrl(),
            close_find_replace: Keycode::Esc.to_key(),
            request_completions: Keycode::Space.to_key().with_ctrl(),
            // I know it's stupid, but at this point I am out of keys on under my left hand
            //  normal people will use context options anyway
            reformat: Keycode::Char('l').to_key().with_ctrl(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FileTree {
    pub toggle_hidden_files: Key,
}

impl Default for FileTree {
    fn default() -> Self {
        FileTree {
            toggle_hidden_files: Keycode::Char('h').to_key().with_ctrl(),
        }
    }
}

pub type ConfigRef = Arc<Config>;

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
