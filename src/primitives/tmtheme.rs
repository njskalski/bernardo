use std::fmt::{Debug, Formatter};
use syntect::highlighting::{Highlighter, ThemeSet};
use serde::{Serialize, Deserialize};
use syntect::parsing::Scope;
use crate::primitives::color::Color;

impl Into<Color> for syntect::highlighting::Color {
    fn into(self) -> Color {
        Color::new(self.r, self.g, self.b)
    }
}

#[derive(Deserialize, Serialize)]
pub struct TmTheme {
    pub tm: ThemeSet,
}

impl TmTheme {
    pub fn color_for_name(&self, name: &str) -> Option<Color> {
        //TODO
        let tm = self.tm.themes.get("base16-eighties.dark")?;
        let highlighter = Highlighter::new(tm);

        let scope = Scope::new(name).ok()?;
        let style = highlighter.style_for_stack(&[scope]);

        Some(style.foreground.into())
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
        TmTheme {
            tm: ThemeSet::load_defaults(),
        }
    }
}