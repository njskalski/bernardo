use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Range;
use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use log::{debug, error, warn};
use ropey::Rope;
use tree_sitter::{InputEdit, Language, Parser, Point, Query, QueryCursor, QueryError};
#[allow(unused_imports)]
use tree_sitter_cpp::*;

use crate::tsw::lang_id::LangId;
use crate::tsw::language_set::LanguageSet;
use crate::tsw::parsing_tuple::ParsingTuple;
use crate::tsw::rope_wrappers::RopeWrapper;
use crate::unpack_or_e;

static EMPTY_SLICE: [u8; 0] = [0; 0];

lazy_static! {
    static ref TREE_SITTER_BASH_HIGHLIGHT_QUERY : &'static str = tree_sitter_bash::HIGHLIGHT_QUERY;


    // I have no idea how I came up with this
    static ref TREE_SITTER_CPP_HIGHLIGHT_QUERY: String = include_str!("../../third-party/nvim-treesitter/queries/c/highlights.scm")
        .to_owned()
        + include_str!("../../third-party/nvim-treesitter/queries/cpp/highlights.scm");

    static ref TREE_SITTER_RUST_INDENT_QUERY: String = include_str!("../../third-party/nvim-treesitter/queries/rust/indents.scm")
        .to_owned();

    static ref OLD_TREE_SITTER_GOLANG_HIGHLIGHT_QUERY_STUPID_LINKER :&'static str = tree_sitter_go::HIGHLIGHTS_QUERY;

    static ref TREE_SITTER_GOLANG_HIGHLIGHT_QUERY: String = include_str!("../../third-party/nvim-treesitter/queries/go/highlights.scm").to_owned();

    static ref TREE_SITTER_PYTHON_HIGHLIGHT_QUERY_STUPID_LINKER: &'static str = tree_sitter_python::HIGHLIGHTS_QUERY;

    static ref TREE_SITTER_PYTHON_HIGHLIGHT_QUERY: String = include_str!("../../third-party/nvim-treesitter/queries/python/highlights.scm").to_owned();

}

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

pub fn pack_rope_with_callback<'a>(rope: &'a Rope) -> Box<dyn FnMut(usize, Point) -> &'a [u8] + 'a> {
    return Box::new(move |offset: usize, point: Point| {
        debug!("request to parse point {:?}, offset {:?}", point, offset);

        if offset >= rope.len_bytes() {
            // debug!("byte offset beyond rope length: {} >= {}", offset, rope.len_bytes());
            return &EMPTY_SLICE;
        }

        let point_from_offset = match byte_offset_to_point(rope, offset) {
            Some(point) => point,
            None => return &EMPTY_SLICE,
        };
        if point != point_from_offset {
            error!(
                "byte offset diverted from point. Point is {},{} and offset {}:{},{}",
                point.column, point.row, offset, point_from_offset.column, point_from_offset.row
            );
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
    });
}

extern "C" {
    fn tree_sitter_bash() -> Language;

    fn tree_sitter_c() -> Language;
    fn tree_sitter_cpp() -> Language;
    fn tree_sitter_html() -> Language;
    fn tree_sitter_go() -> Language;
    fn tree_sitter_rust() -> Language;

    fn tree_sitter_python() -> Language;
}

#[derive(Debug)]
pub struct TreeSitterWrapper {
    languages: HashMap<LangId, Language>,
}

impl TreeSitterWrapper {
    pub fn new(ls: LanguageSet) -> TreeSitterWrapper {
        let mut languages = HashMap::<LangId, Language>::new();

        if ls.bash {
            let language_bash = unsafe { tree_sitter_bash() };
            languages.insert(LangId::BASH, language_bash);
        }

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

        if ls.python3 {
            let language_python3 = unsafe { tree_sitter_python() };
            languages.insert(LangId::PYTHON3, language_python3);
        }

        if ls.go {
            let language_go = unsafe { tree_sitter_go() };
            languages.insert(LangId::GO, language_go);
        }

        if ls.rust {
            let language_rust = unsafe { tree_sitter_rust() };
            languages.insert(LangId::RUST, language_rust);
        }

        TreeSitterWrapper { languages }
    }

    pub fn highlight_query(&self, lang_id: LangId) -> Option<&str> {
        #[allow(unreachable_patterns)]
        match lang_id {
            LangId::BASH => Some(tree_sitter_bash::HIGHLIGHT_QUERY),
            LangId::C => Some(tree_sitter_c::HIGHLIGHT_QUERY),
            LangId::CPP => Some(TREE_SITTER_CPP_HIGHLIGHT_QUERY.as_str()),
            LangId::HTML => Some(tree_sitter_html::HIGHLIGHTS_QUERY),
            LangId::GO => Some(&TREE_SITTER_GOLANG_HIGHLIGHT_QUERY),
            LangId::PYTHON3 => Some(&TREE_SITTER_PYTHON_HIGHLIGHT_QUERY),
            LangId::RUST => Some(tree_sitter_rust::HIGHLIGHTS_QUERY),
            _ => None,
        }
    }

    pub fn indent_query(&self, lang_id: LangId) -> Option<&str> {
        #[allow(unreachable_patterns)]
        match lang_id {
            LangId::RUST => Some(TREE_SITTER_RUST_INDENT_QUERY.as_str()),
            _ => None,
        }
    }

    // This should be called on loading a file. On update, ParserAndTree struct should be used.
    pub fn new_parse(&self, lang_id: LangId) -> Option<ParsingTuple> {
        let language = self.languages.get(&lang_id)?;
        let highlight_query_str = self.highlight_query(lang_id)?;
        let mut parser = Parser::new();
        match parser.set_language(language) {
            Ok(_) => {}
            Err(e) => {
                error!("failed setting language: {}", e);
                return None;
            }
        };

        let highlight_query = unpack_or_e!(
            Query::new(language, highlight_query_str).ok(),
            None,
            "failed to compile highlight query"
        );

        let indent_query = match self.indent_query(lang_id) {
            None => None,
            Some(query_string) => match Query::new(language, query_string) {
                Ok(q) => Some(q),
                Err(e) => {
                    error!("failed compiling indent query: {}", e);
                    None
                }
            },
        };

        let id_to_name: Vec<Arc<String>> = highlight_query.capture_names().iter().map(|cn| Arc::new(cn.to_string())).collect();

        Some(ParsingTuple {
            tree: None,
            lang_id,
            parser: Arc::new(RwLock::new(parser)),
            language: language.clone(),
            highlight_query: Arc::new(highlight_query),
            indent_query: indent_query.map(Arc::new),
            id_to_name: Arc::new(id_to_name),
        })
    }
}

#[derive(Clone, Debug)]
pub struct HighlightItem {
    pub char_begin: usize,
    pub char_end: usize,
    pub identifier: Arc<String>,
}

impl ParsingTuple {
    // TODO I would prefer it to be an iterator, but I have no time to fix it.
    pub fn highlight_iter<'a>(&'a self, rope: &'a ropey::Rope, char_range_op: Option<Range<usize>>) -> Option<Vec<HighlightItem>> {
        if self.tree.is_none() {
            return None;
        }

        let mut cursor = QueryCursor::new();

        if let Some(char_range) = char_range_op {
            let begin_byte = rope.try_char_to_byte(char_range.start).ok()?;
            let end_byte = rope.try_char_to_byte(char_range.end).ok()?;

            cursor.set_byte_range(begin_byte..end_byte);
        };

        let _query_captures: Vec<_> = cursor
            .captures(&self.highlight_query, self.tree.as_ref()?.root_node(), RopeWrapper(&rope))
            .collect();
        let query_matches = cursor.matches(&self.highlight_query, self.tree.as_ref()?.root_node(), RopeWrapper(&rope));

        let mut results: Vec<HighlightItem> = vec![];
        for m in query_matches {
            if m.captures.len() != 1 {
                warn!("unexpected number of captures (expected 1, got {})", m.captures.len());
            }

            for c in m.captures {
                let begin_char = rope.try_byte_to_char(c.node.start_byte()).ok()?;
                let end_char = rope.try_byte_to_char(c.node.end_byte()).ok()?;

                let name = self.id_to_name.get(c.index as usize)?;

                results.push(HighlightItem {
                    char_begin: begin_char,
                    char_end: end_char,
                    identifier: name.clone(),
                })
            }
        }

        debug!("highlight result size = {}", results.len());

        Some(results)
    }

    pub fn try_reparse(&mut self, rope: &ropey::Rope) -> bool {
        let mut callback = pack_rope_with_callback(rope);
        let mut parser = unpack_or_e!(self.parser.try_write().ok(), false, "failed to lock parser");

        debug!("doing reparse, tree = {:?}", self.tree);
        let tree = unpack_or_e!(parser.parse_with(&mut callback, self.tree.as_ref()), false, "failed parse");

        self.tree = Some(tree);

        let query_captures: Vec<_> = QueryCursor::new()
            .captures(&self.highlight_query, self.tree.as_ref().unwrap().root_node(), RopeWrapper(&rope))
            .collect();

        for (_idx, m) in QueryCursor::new()
            .matches(&self.highlight_query, self.tree.as_ref().unwrap().root_node(), RopeWrapper(&rope))
            .enumerate()
        {
            for (_cidx, c) in m.captures.iter().enumerate() {
                let name = &self.id_to_name[c.index as usize];
                debug!(
                    "m[{}]c[{}] : [{}:{}) = {}",
                    _idx,
                    _cidx,
                    c.node.start_byte(),
                    c.node.end_byte(),
                    name,
                );
            }
        }

        true
    }

    pub fn update_parse_on_insert(&mut self, rope: &ropey::Rope, char_idx_begin: usize, char_idx_end: usize) -> bool {
        if rope.len_chars() < char_idx_begin {
            error!("rope.len_chars() < char_idx_begin: {} >= {}", rope.len_chars(), char_idx_begin);
            return false;
        }

        let start_byte = match rope.try_char_to_byte(char_idx_begin) {
            Ok(byte) => byte,
            _ => return false,
        };

        let new_end_byte = match rope.try_char_to_byte(char_idx_end) {
            Ok(byte) => byte,
            _ => return false,
        };

        let start_point = match byte_offset_to_point(&rope, start_byte) {
            Some(point) => point,
            None => return false,
        };

        let new_end_point = match byte_offset_to_point(&rope, new_end_byte) {
            Some(point) => point,
            None => return false,
        };

        let input_edit = InputEdit {
            start_byte,
            old_end_byte: start_byte,
            new_end_byte,
            start_position: start_point,
            old_end_position: start_point,
            new_end_position: new_end_point,
        };

        self.tree.as_mut().map(|tree| tree.edit(&input_edit));
        self.try_reparse(rope)
    }

    pub fn update_parse_on_delete(&mut self, rope: &ropey::Rope, char_idx_begin: usize, char_idx_end: usize) -> bool {
        if char_idx_begin >= char_idx_end {
            error!("char_idx_begin >= char_idx_end: {} >= {}", char_idx_begin, char_idx_end);
            return false;
        }

        if rope.len_chars() < char_idx_begin {
            error!("rope.len_chars() < char_idx_begin: {} >= {}", rope.len_chars(), char_idx_begin);
            return false;
        }

        let start_byte = match rope.try_char_to_byte(char_idx_begin) {
            Ok(byte) => byte,
            _ => return false,
        };

        let old_end_byte = match rope.try_char_to_byte(char_idx_end) {
            Ok(byte) => byte,
            _ => return false,
        };

        let start_point = match byte_offset_to_point(&rope, start_byte) {
            Some(point) => point,
            None => return false,
        };

        let old_end_point = match byte_offset_to_point(&rope, old_end_byte) {
            Some(point) => point,
            None => return false,
        };

        let input_edit = InputEdit {
            start_byte,
            old_end_byte,
            new_end_byte: start_byte,
            start_position: start_point,
            old_end_position: old_end_point,
            new_end_position: start_point,
        };

        self.tree.as_mut().map(|tree| tree.edit(&input_edit));
        self.try_reparse(rope)
    }
}
