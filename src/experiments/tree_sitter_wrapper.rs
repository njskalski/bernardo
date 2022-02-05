use std::collections::HashMap;

use log::{error, warn};
use tree_sitter::{Language, LanguageError, Parser, Tree};
use tree_sitter_highlight::HighlightConfiguration;

pub const LANGID_C: &'static str = "c";
pub const LANGID_CPP: &'static str = "cpp";
pub const LANGID_HTML: &'static str = "html";
pub const LANGID_ELM: &'static str = "elm";
pub const LANGID_GO: &'static str = "go";
pub const LANGID_RUST: &'static str = "rust";

extern "C" {
    fn tree_sitter_c() -> Language;
    fn tree_sitter_cpp() -> Language;
    fn tree_sitter_html() -> Language;
    fn tree_sitter_elm() -> Language;
    fn tree_sitter_go() -> Language;
    fn tree_sitter_rust() -> Language;
}

pub struct TreeSitterWrapper {
    languages: HashMap<&'static str, Language>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct LanguageSet {
    pub c: bool,
    pub cpp: bool,
    pub elm: bool,
    pub go: bool,
    pub html: bool,
    pub rust: bool,
}

impl LanguageSet {
    pub fn full() -> Self {
        LanguageSet {
            c: true,
            cpp: true,
            elm: true,
            go: true,
            html: true,
            rust: true,
        }
    }
}

impl TreeSitterWrapper {
    pub fn new(ls: LanguageSet) -> TreeSitterWrapper {
        let mut languages = HashMap::<&'static str, Language>::new();

        if ls.c {
            let language_c = unsafe { tree_sitter_c() };
            languages.insert(LANGID_C, language_c);
        }

        if ls.cpp {
            let language_cpp = unsafe { tree_sitter_cpp() };
            languages.insert(LANGID_CPP, language_cpp);
        }

        if ls.html {
            let language_html = unsafe { tree_sitter_html() };
            languages.insert(LANGID_HTML, language_html);
        }

        if ls.elm {
            let language_elm = unsafe { tree_sitter_elm() };
            languages.insert(LANGID_ELM, language_elm);
        }

        if ls.go {
            let language_go = unsafe { tree_sitter_go() };
            languages.insert(LANGID_GO, language_go);
        }

        if ls.rust {
            let language_rust = unsafe { tree_sitter_rust() };
            languages.insert(LANGID_RUST, language_rust);
        }

        TreeSitterWrapper {
            languages
        }
    }
}