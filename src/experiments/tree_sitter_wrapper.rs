use std::collections::HashMap;
use std::f32::consts::E;
use std::path::Path;

use log::{debug, error, warn};
use ropey::Rope;
use tree_sitter::{Language, LanguageError, Parser, Point, Tree};
use tree_sitter_highlight::HighlightConfiguration;

use crate::text::buffer::Buffer;

pub type TreeSitterCallback<T: AsRef<[u8]>> = FnMut(usize, Point) -> T;

static EMPTY_SLICE: [u8; 0] = [u8; 0];

pub fn pack_rope_with_callback<'a>(rope: &'a Rope) -> fn(usize, Point) -> &[u8] {
    return |offset: usize, point: Point| {
        if offset >= rope.len_bytes() {
            return &EMPTY_SLICE
        }

        // next several lines are just a sanity check
        let char_idx = match rope.try_byte_to_char(offset) {
            Some(idx) => idx,
            None => return &EMPTY_SLICE,
        };
        let line_idx = match rope.try_char_to_line(char_idx) {
            Some(idx) => idx,
            None => return &EMPTY_SLICE,
        };
        let line_begin_idx = match rope.try_line_to_char(line_idx) {
            Some(idx) => idx,
            None => return &EMPTY_SLICE,
        };
        let column_idx = char_idx - line_begin_idx;
        if point.row == line_idx || point.column == column_idx {
            debug!("byte offset diverted from point. Point is {},{} and offset {}:{},{}",
                point.column, point.row, offset, column_idx, line_idx);
        }
        // end of sanity check

        let (bytes, _, _, _) = rope.chunk_at_byte(offset);
        bytes.as_bytes()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum LangId {
    C,
    CPP,
    HTML,
    ELM,
    GO,
    RUST,
}

extern "C" {
    fn tree_sitter_c() -> Language;
    fn tree_sitter_cpp() -> Language;
    fn tree_sitter_html() -> Language;
    fn tree_sitter_elm() -> Language;
    fn tree_sitter_go() -> Language;
    fn tree_sitter_rust() -> Language;
}

pub struct TreeSitterWrapper {
    languages: HashMap<LangId, Language>,
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

pub struct ParserAndTree {
    pub parser: Parser,
    pub tree: Tree,
    pub lang: LangId,
}

impl TreeSitterWrapper {
    pub fn new(ls: LanguageSet) -> TreeSitterWrapper {
        let mut languages = HashMap::<LangId, Language>::new();

        if ls.c {
            let language_c = unsafe { tree_sitter_c() };
            languages.insert(LangId::C, language_c);
        }

        if ls.cpp {
            let language_cpp = unsafe { tree_sitter_cpp() };
            languages.insert(LangId::CPP, language_cpp);
        }

        if ls.html {
            let language_html = unsafe { tree_sitter_html() };
            languages.insert(LangId::HTML, language_html);
        }

        if ls.elm {
            let language_elm = unsafe { tree_sitter_elm() };
            languages.insert(LangId::ELM, language_elm);
        }

        if ls.go {
            let language_go = unsafe { tree_sitter_go() };
            languages.insert(LangId::GO, language_go);
        }

        if ls.rust {
            let language_rust = unsafe { tree_sitter_rust() };
            languages.insert(LangId::RUST, language_rust);
        }

        TreeSitterWrapper {
            languages
        }
    }

    // This should be called on loading a file. On update, ParserAndTree struct should be used.
    // pub fn new_parse(&self, langId: LangId, buffer: &dyn Buffer) -> Result<ParserAndTree, ()> {
    //     let language = self.languages.get(&langId).ok_or(Err(()))?;
    //     let mut parser = Parser::new();
    //     parser.set_language(language.clone())?;
    //
    //     // let tree = parser.parse_with()
    // }
}
