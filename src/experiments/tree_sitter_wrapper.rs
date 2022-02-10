use std::collections::HashMap;
use std::f32::consts::E;
use std::path::Path;

use log::{debug, error, warn};
use ropey::Rope;
use tree_sitter::{Language, LanguageError, Parser, Point, Tree};
use tree_sitter_highlight::HighlightConfiguration;

use crate::text::buffer::Buffer;

static EMPTY_SLICE: [u8; 0] = [0; 0];

pub fn byte_offset_to_point(rope: &Rope, byte_offset: usize) -> Option<Point> {
    let char_idx = rope.try_byte_to_char(byte_offset).ok()?;
    let line_idx = rope.try_char_to_line(char_idx).ok()?;
    let line_begin_char_idx = rope.try_line_to_char(line_idx).ok()?;

    // some paranoia
    if char_idx < line_begin_char_idx {
        None
    } else {
        let column_idx = char_idx - line_begin_char_idx;
        Some(Point::new(line_idx, column_idx))
    }
}

pub fn pack_rope_with_callback<'a>(rope: &'a Rope) -> Box<FnMut(usize, Point) -> &'a [u8] + 'a> {
    return Box::new(move |offset: usize, point: Point| {
        if offset >= rope.len_bytes() {
            debug!("byte offset beyond rope length: {} >= {}", offset, rope.len_bytes());
            return &EMPTY_SLICE
        }

        let point_from_offset = match byte_offset_to_point(rope, offset) {
            Some(point) => point,
            None => return &EMPTY_SLICE,
        };
        if point != point_from_offset {
            error!("byte offset diverted from point. Point is {},{} and offset {}:{},{}",
                point.column, point.row, offset, point_from_offset.column, point_from_offset.row);
        }
        // end of sanity check

        //(chunk, chunk_byte_idx, chunk_char_idx, chunk_line_idx).
        let (chunk, chunk_byte_idx, _, _) = rope.chunk_at_byte(offset);
        let chunk_as_bytes = chunk.as_bytes();
        // now chunk most probably begins BEFORE our offset. We need to offset for the offset.

        debug_assert!(offset >= chunk_byte_idx); //TODO add some non-panicking failsafe.
        let cut_from_beginning = offset - chunk_byte_idx;
        let result = &chunk_as_bytes[cut_from_beginning..];

        debug!("parser reading bytes [{}-{}) |{}|", offset, offset + result.len(), result.len());

        result
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Debug)]
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
    pub fn new_parse(&self, langId: LangId, buffer: &dyn Buffer) -> Option<(Parser, Tree)> {
        let language = self.languages.get(&langId)?;
        let mut parser = Parser::new();
        parser.set_language(language.clone());

        let mut callback = buffer.callback_for_parser();
        let tree = parser.parse_with(&mut callback, None)?;

        Some(
            (
                parser,
                tree,
            )
        )
    }
}
