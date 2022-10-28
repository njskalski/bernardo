use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Range;
use std::rc::Rc;

use log::{error, warn};
use ropey::Rope;
use tree_sitter::{InputEdit, Language, Parser, Point, Query, QueryCursor};

use crate::text::text_buffer::TextBuffer;
use crate::tsw::lang_id::LangId;
use crate::tsw::language_set::LanguageSet;
use crate::tsw::parsing_tuple::ParsingTuple;
use crate::tsw::rope_wrappers::RopeWrapper;

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

pub fn pack_rope_with_callback<'a>(rope: &'a Rope) -> Box<dyn FnMut(usize, Point) -> &'a [u8] + 'a> {
    return Box::new(move |offset: usize, point: Point| {
        if offset >= rope.len_bytes() {
            // debug!("byte offset beyond rope length: {} >= {}", offset, rope.len_bytes());
            return &EMPTY_SLICE;
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

        // debug!("parser reading bytes [{}-{}) |{}|", offset, offset + result.len(), result.len());

        result
    });
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

    pub fn highlight_query(&self, lang_id: LangId) -> Option<&'static str> {
        match lang_id {
            LangId::C => Some(tree_sitter_c::HIGHLIGHT_QUERY),
            LangId::CPP => Some(tree_sitter_cpp::HIGHLIGHT_QUERY),
            LangId::HTML => Some(tree_sitter_html::HIGHLIGHT_QUERY),
            LangId::ELM => Some(tree_sitter_elm::HIGHLIGHTS_QUERY),
            LangId::GO => Some(tree_sitter_go::HIGHLIGHT_QUERY),
            LangId::RUST => Some(tree_sitter_rust::HIGHLIGHT_QUERY),
            _ => None
        }
    }

    // This should be called on loading a file. On update, ParserAndTree struct should be used.
    pub fn new_parse(&self, lang_id: LangId) -> Option<ParsingTuple> {
        let language = self.languages.get(&lang_id)?;
        let highlight_query = self.highlight_query(lang_id)?;
        let mut parser = Parser::new();
        match parser.set_language(language.clone()) {
            Ok(_) => {}
            Err(e) => {
                error!("failed setting language: {}", e);
                return None;
            }
        };

        let query = match Query::new(
            *language,
            highlight_query) {
            Ok(query) => query,
            Err(e) => {
                error!("failed to compile query {}", e);
                return None;
            }
        };

        let id_to_name: Vec<Rc<String>> = query.capture_names().iter().map(|cn| {
            Rc::new(cn.to_owned())
        }).collect();

        Some(
            ParsingTuple {
                tree: None,
                lang_id,
                parser: Rc::new(RefCell::new(parser)),
                language: language.clone(),
                highlight_query: Rc::new(query),
                id_to_name: Rc::new(id_to_name),
            }
        )
    }
}

#[derive(Debug)]
pub struct HighlightItem {
    pub char_begin: usize,
    pub char_end: usize,
    pub identifier: Rc<String>,
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

        let query_matches = cursor.matches(
            &self.highlight_query,
            self.tree.as_ref()?.root_node(),
            RopeWrapper(&rope),
        );

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

        Some(results)
    }

    pub fn try_reparse(&mut self, rope: &ropey::Rope) -> bool {
        let mut callback = rope.callback_for_parser();
        //TODO borrow_mut => try_borrow_mut
        let tree = match self.parser.borrow_mut().parse_with(
            &mut callback, self.tree.as_ref()) {
            Some(t) => t,
            None => {
                error!("failed parse");
                return false;
            }
        };

        self.tree = Some(tree);

        // for (_idx, m) in QueryCursor::new().matches(
        //     &self.highlight_query,
        //     self.tree.as_ref().unwrap().root_node(),
        //     RopeWrapper(&rope),
        // ).enumerate() {
        //     for (_cidx, c) in m.captures.iter().enumerate() {
        //         // let name = &self.id_to_name[c.index as usize];
        //         // debug!("m[{}]c[{}] : [{}:{}) = {}",
        //         //     _idx, _cidx,
        //         //     c.node.start_byte(),
        //         //     c.node.end_byte(),
        //         //     name,
        //         //     );
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