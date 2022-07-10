use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

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

    cache: RefCell<HashMap<String, Color>>,
}


impl TmTheme {
    pub fn color_for_name(&self, name: &str) -> Option<Color> {
        if let Some(color) = self.cache.borrow().get(&*name) {
            return Some(*color);
        }

        // this is slow, so I cache
        let highlighter = Highlighter::new(&self.theme);

        let color = match Scope::new(name) {
            Ok(scope) => highlighter.style_for_stack(&[scope]).foreground,
            Err(_) => highlighter.get_default().foreground,
        };


        self.cache.borrow_mut().insert(name.to_string(), color.into());
        Some(color.into())
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
            cache: RefCell::new(HashMap::new()),
        }
    }
}