use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::RwLock;

use log::{debug, error};
use syntect::highlighting::{Highlighter, ThemeSet};
use syntect::parsing::Scope;

use crate::primitives::color::Color;

impl Into<Color> for syntect::highlighting::Color {
    fn into(self) -> Color {
        Color::new(self.r, self.g, self.b)
    }
}

pub struct TmTheme {
    theme: syntect::highlighting::Theme,
    cache: RwLock<HashMap<String, Color>>,
}

impl TmTheme {
    pub fn name_to_color(&self, name: &str) -> Option<Color> {
        debug!("name_to_color({})", name);

        match self.cache.try_read() {
            Ok(cache) => {
                if let Some(color) = cache.get(&*name) {
                    return Some(*color);
                } else {
                    debug!("cache miss for {}", name);
                }
            }
            Err(_) => {
                error!("failed to acquire cache lock");
            }
        }

        // this is slow, so I cache
        let highlighter = Highlighter::new(&self.theme);

        let color = match Scope::new(name) {
            Ok(scope) => highlighter.style_for_stack(&[scope]).foreground,
            Err(_) => highlighter.get_default().foreground,
        };

        match self.cache.try_write() {
            Ok(mut cache) => {
                cache.insert(name.to_string(), color.into());
            }
            Err(_) => {
                error!("failed to cache color_for_name");
            }
        }

        Some(color.into())
    }

    pub fn get_cache(&self) -> Option<HashMap<String, Color>> {
        self.cache.read().ok().map(|lock| {
            let clone: HashMap<String, Color> = lock.clone();
            clone
        })
    }

    pub fn new(syntect_theme: syntect::highlighting::Theme) -> TmTheme {
        TmTheme {
            theme: syntect_theme,
            cache: Default::default(),
        }
    }
}

impl PartialEq for TmTheme {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for TmTheme {}

impl Debug for TmTheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "syntect::ThemeSet")
    }
}

impl Default for TmTheme {
    fn default() -> Self {
        let tm = ThemeSet::load_defaults().themes.get("base16-eighties.dark").unwrap().clone();

        TmTheme {
            theme: tm,
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl Clone for TmTheme {
    fn clone(&self) -> Self {
        TmTheme {
            theme: self.theme.clone(),
            cache: Default::default(),
        }
    }
}
