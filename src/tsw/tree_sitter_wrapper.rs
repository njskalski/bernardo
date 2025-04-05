use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::primitives::printable::Printable;
use lazy_static::lazy_static;
use log::{debug, error, warn};
use ropey::Rope;
use streaming_iterator::StreamingIterator;
use tree_sitter::{InputEdit, Language, Parser, Point, Query, QueryCursor, QueryError};
use tree_sitter_language::LanguageFn;
use tree_sitter_loader::{CompileConfig, Config};
// #[allow(unused_imports)]
// use tree_sitter_cpp::*;

use crate::tsw::lang_id::LangId;
use crate::tsw::language_set::LanguageSet;
use crate::tsw::parsing_tuple::ParsingTuple;
use crate::tsw::rope_wrappers::RopeWrapper;
use crate::unpack_or_e;

static EMPTY_SLICE: [u8; 0] = [0; 0];

lazy_static! {
    // static ref TREE_SITTER_TYPESCRIPT_HIGHLIGHT_QUERY: String = include_str!("../../third-party/tree-sitter-typescript/queries/highlights.scm")
    //     .to_owned()
    //     + include_str!("../../third-party/tree-sitter-typescript/queries/locals.scm")
    //     + include_str!("../../third-party/tree-sitter-typescript/queries/tags.scm");

   static ref TREE_SITTER_BASH_HIGHLIGHT_QUERY : String = include_str!("../../third-party/nvim-treesitter/queries/bash/highlights.scm")
        .to_owned();

    // I have no idea how I came up with this
    static ref TREE_SITTER_CPP_HIGHLIGHT_QUERY: String = include_str!("../../third-party/nvim-treesitter/queries/c/highlights.scm")
        .to_owned()
        + include_str!("../../third-party/nvim-treesitter/queries/cpp/highlights.scm");

    static ref TREE_SITTER_RUST_HIGHLIGHT_QUERY: String = include_str!("../../third-party/nvim-treesitter/queries/rust/highlights.scm")
        .to_owned();

    static ref TREE_SITTER_GOLANG_HIGHLIGHT_QUERY: String = include_str!("../../third-party/nvim-treesitter/queries/go/highlights.scm").to_owned();

    static ref TREE_SITTER_PYTHON_HIGHLIGHT_QUERY: String = include_str!("../../third-party/nvim-treesitter/queries/python/highlights.scm").to_owned();

    static ref TREE_SITTER_TYPESCRIPT_HIGHLIGHT_QUERY: String = include_str!("../../third-party/nvim-treesitter/queries/ecma/highlights.scm")
        .to_owned();
        // + include_str!("../../third-party/nvim-treesitter/queries/typescript/highlights.scm")
        // + include_str!("../../third-party/nvim-treesitter/queries/jsx/highlights.scm");

    static ref TREE_SITTER_HASKELL_HIGHLIGHT_QUERY: String = include_str!("../../third-party/nvim-treesitter/queries/haskell/highlights.scm").to_owned();

    static ref TREE_SITTER_TOML_HIGHLIGHT_QUERY : String = include_str!("../../third-party/nvim-treesitter/queries/toml/highlights.scm").to_owned();

    static ref TREE_SITTER_JAVA_HIGHLIGHT_QUERY : String = include_str!("../../third-party/nvim-treesitter/queries/java/highlights.scm").to_owned();

    static ref TREE_SITTER_HTML_HIGHLIGHT_QUERY : String = include_str!("../../third-party/nvim-treesitter/queries/html/highlights.scm").to_owned();

    static ref TREE_SITTER_YAML_HIGHLIGHT_QUERY : String = include_str!("../../third-party/nvim-treesitter/queries/yaml/highlights.scm").to_owned();
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

#[derive(Debug)]
pub struct TreeSitterTuple {
    pub lang_id: LangId,
    pub language: Language,
    // pub highlighter_query:
}

fn load_languages_from_submodules() -> HashMap<LangId, TreeSitterTuple> {
    let language_to_paths: Vec<(LangId, &'static str, Language)> = vec![
        (LangId::BASH, "../../third-party/tree_sitter_bash", LANGUAGE_BASH.into()),
        (LangId::C, "../../third-party/tree_sitter_c", LANGUAGE_C.into()),
        (LangId::CPP, "../../third-party/tree_sitter_cpp", LANGUAGE_CPP.into()),
        (LangId::HASKELL, "../../third-party/tree_sitter_haskell", LANGUAGE_HASKELL.into()),
        (LangId::HTML, "../../third-party/tree_sitter_html", LANGUAGE_HTML.into()),
        (LangId::JAVA, "../../third-party/tree_sitter_java", LANGUAGE_JAVA.into()),
        (
            LangId::JAVASCRIPT,
            "../../third-party/tree_sitter_javascript",
            LANGUAGE_JAVASCRIPT.into(),
        ),
        (
            LangId::TYPESCRIPT,
            "../../third-party/tree_sitter_typescript",
            LANGUAGE_TYPESCRIPT.into(),
        ),
        (LangId::GO, "../../third-party/tree_sitter_go", LANGUAGE_GO.into()),
        (LangId::PYTHON3, "../../third-party/tree_sitter_python", LANGUAGE_PYTHON.into()),
        (LangId::RUST, "../../third-party/tree_sitter_rust", LANGUAGE_RUST.into()),
        (LangId::TOML, "../../third-party/tree_sitter_toml", LANGUAGE_TOML.into()),
        (LangId::YAML, "../../third-party/tree-sitter-yaml", LANGUAGE_YAML.into()),
    ];

    let mut result = HashMap::<LangId, TreeSitterTuple>::new();

    for (lang_id, _, language) in language_to_paths {
        debug_assert!(result.contains_key(&lang_id) == false, "duplicate language id: {}", lang_id);

        let tuple = TreeSitterTuple { lang_id, language };

        result.insert(lang_id, tuple);
    }

    result
}

#[derive(Debug)]
pub struct TreeSitterWrapper {
    languages: HashMap<LangId, TreeSitterTuple>,
}

extern "C" {
    fn tree_sitter_bash() -> *const ();
    fn tree_sitter_c() -> *const ();
    fn tree_sitter_cpp() -> *const ();

    fn tree_sitter_haskell() -> *const ();
    fn tree_sitter_html() -> *const ();

    fn tree_sitter_java() -> *const ();
    fn tree_sitter_javascript() -> *const ();

    fn tree_sitter_typescript() -> *const ();
    fn tree_sitter_go() -> *const ();

    fn tree_sitter_python() -> *const ();
    fn tree_sitter_rust() -> *const ();
    fn tree_sitter_toml() -> *const ();

    fn tree_sitter_yaml() -> *const ();
}

pub const LANGUAGE_BASH: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_bash) };
pub const LANGUAGE_C: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_c) };
pub const LANGUAGE_CPP: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_cpp) };

pub const LANGUAGE_HASKELL: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_haskell) };
pub const LANGUAGE_HTML: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_html) };
pub const LANGUAGE_JAVA: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_java) };
pub const LANGUAGE_JAVASCRIPT: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_javascript) };
pub const LANGUAGE_TYPESCRIPT: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_typescript) };
pub const LANGUAGE_GO: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_go) };
pub const LANGUAGE_PYTHON: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_python) };
pub const LANGUAGE_RUST: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_rust) };
pub const LANGUAGE_TOML: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_toml) };

pub const LANGUAGE_YAML: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_yaml) };

impl TreeSitterWrapper {
    pub fn new(ls: LanguageSet) -> TreeSitterWrapper {
        let loaded_languages = load_languages_from_submodules();
        TreeSitterWrapper {
            languages: loaded_languages,
        }
    }

    pub fn highlight_query(&self, lang_id: LangId) -> Option<&str> {
        #[allow(unreachable_patterns)]
        match lang_id {
            LangId::BASH => Some(&TREE_SITTER_BASH_HIGHLIGHT_QUERY),
            LangId::C => Some(&TREE_SITTER_CPP_HIGHLIGHT_QUERY),
            LangId::CPP => Some(&TREE_SITTER_CPP_HIGHLIGHT_QUERY),
            LangId::HASKELL => Some(&TREE_SITTER_HASKELL_HIGHLIGHT_QUERY),
            LangId::JAVA => Some(&TREE_SITTER_JAVA_HIGHLIGHT_QUERY),
            LangId::JAVASCRIPT => Some(&TREE_SITTER_TYPESCRIPT_HIGHLIGHT_QUERY),
            LangId::GO => Some(&TREE_SITTER_GOLANG_HIGHLIGHT_QUERY),
            LangId::PYTHON3 => Some(&TREE_SITTER_PYTHON_HIGHLIGHT_QUERY),
            LangId::RUST => Some(&TREE_SITTER_RUST_HIGHLIGHT_QUERY),
            LangId::TOML => Some(&TREE_SITTER_TOML_HIGHLIGHT_QUERY),
            LangId::TYPESCRIPT => Some(&TREE_SITTER_TYPESCRIPT_HIGHLIGHT_QUERY),
            LangId::HTML => Some(&TREE_SITTER_HTML_HIGHLIGHT_QUERY),
            LangId::YAML => Some(&TREE_SITTER_YAML_HIGHLIGHT_QUERY),
            _ => None,
        }
    }

    pub fn indent_query(&self, lang_id: LangId) -> Option<&str> {
        #[allow(unreachable_patterns)]
        match lang_id {
            LangId::RUST => Some(TREE_SITTER_RUST_HIGHLIGHT_QUERY.as_str()),
            _ => None,
        }
    }

    // This should be called on loading a file. On update, ParserAndTree struct should be used.
    pub fn new_parse(&self, lang_id: LangId) -> Option<ParsingTuple> {
        let tuple = self.languages.get(&lang_id)?;
        let highlight_query_str = self.highlight_query(lang_id)?;
        let mut parser = Parser::new();
        match parser.set_language(&tuple.language) {
            Ok(_) => {}
            Err(e) => {
                error!("failed setting language: {}", e);
                return None;
            }
        };

        let highlight_query = match Query::new(&tuple.language, highlight_query_str) {
            Ok(q) => q,
            Err(e) => {
                error!("failed compiling highlight query: {}", e);
                return None;
            }
        };

        let indent_query = match self.indent_query(lang_id) {
            None => None,
            Some(query_string) => match Query::new(&tuple.language, query_string) {
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
            language: tuple.language.clone(),
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

        // let _query_captures: Vec<_> = cursor
        //     .captures(&self.highlight_query, self.tree.as_ref()?.root_node(), RopeWrapper(&rope));
        //

        let mut query_matches = cursor.matches(&self.highlight_query, self.tree.as_ref()?.root_node(), RopeWrapper(&rope));

        let mut results: Vec<HighlightItem> = vec![];

        while let Some(m) = query_matches.next() {
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

        // #[cfg(debug_assertions)]
        // {
        //     let mut cursor = QueryCursor::new()
        //         .matches(&self.highlight_query, self.tree.as_ref().unwrap().root_node(), RopeWrapper(&rope));
        //
        //     let mut idx: usize = 0;
        //
        //     while let Some(m) = &cursor.next() {
        //         for (_cidx, c) in m.captures.iter().enumerate() {
        //             let name = &self.id_to_name[c.index as usize];
        //             debug!(
        //                 "m[{}]c[{}] : [{}:{}) = {}",
        //                 idx,
        //                 _cidx,
        //                 c.node.start_byte(),
        //                 c.node.end_byte(),
        //                 name,
        //             );
        //         }
        //
        //         idx += 1;
        //     }
        // }

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
