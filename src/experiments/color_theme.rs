use std::collections::HashMap;

use json;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::primitives::color::Color;

pub struct ColorTheme {
    name: HashMap<String, Color>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralCodeTheme {
    pub identifier: Option<Color>,
    pub string_literal: Option<Color>,
    pub type_identifier: Option<Color>,
    pub parenthesis: Option<Color>,
    pub bracket: Option<Color>,
}

impl ColorTheme {
    // pub fn from_json<T>(s: T) -> Option<ColorTheme> where T: Into<&str> {
    //     let j = match json::parse(s.into()) {
    //         Ok(_) => {}
    //         Err(e) => {
    //             debug!("error loading theme: {}", e);
    //             return None;
    //         }
    //     };
    // }
}