use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::rc::Rc;
use std::string::String;

use log::{debug, error, warn};
use ropey::Rope;
use tree_sitter::{InputEdit, Parser, Point, Query, QueryCursor, Tree};
use tree_sitter_highlight::{Highlighter, RopeWrapper};
use unicode_segmentation::UnicodeSegmentation;

use crate::experiments::tree_sitter_wrapper::{byte_offset_to_point, LangId, ParsingTuple};
use crate::text::buffer::Buffer;
use crate::TreeSitterWrapper;

#[derive(Clone, Debug, Default)]
struct Text {
    pub rope: Rope,
    pub parsing: Option<ParsingTuple>,
}

impl Text {
    pub fn with_rope(self, rope: Rope) -> Self {
        Self {
            rope,
            ..self
        }
    }

    pub fn parse(&mut self, tree_sitter: Rc<TreeSitterWrapper>, lang_id: LangId) -> bool {
        if let Some(parsing_tuple) = tree_sitter.new_parse(lang_id, &self.rope) {
            self.parsing = Some(parsing_tuple);
            true
        } else {
            false
        }
    }

    // fn try_reparse_after_tree_update(&mut self) {
    //     let mut callback = self.rope.callback_for_parser();
    //     if let Some(parsing) = self.parsing.as_mut() {
    //         if !parsing.try_reparse(&self.rope) {
    //             error!("try reparse failed");
    //             return;
    //         }
    //
    //         let query = match Query::new(
    //             parsing.language,
    //             parsing.lang_highlight_query) {
    //             Ok(query) => query,
    //             Err(e) => {
    //                 error!("failed to compile query {}", e);
    //                 return;
    //             }
    //         };
    //
    //         for m in QueryCursor::new().matches(
    //             &query,
    //             //TODO this unwrap is OK because above I do early exit if try_parse fails.
    //             parsing.tree.as_ref().unwrap().root_node(),
    //             RopeWrapper(&self.rope),
    //         ) {
    //             debug!("qm : {:?}", m);
    //         }
    //     }
    // }
}

#[derive(Clone, Debug)]
pub struct BufferState {
    tree_sitter: Rc<TreeSitterWrapper>,

    text: Text,

    history: Vec<Text>,
    forward_history: Vec<Text>,

    lang_id: Option<LangId>,

    file_path: Option<PathBuf>,
}

impl BufferState {
    pub fn new(tree_sitter: Rc<TreeSitterWrapper>) -> BufferState {
        BufferState {
            tree_sitter,
            text: Text::default(),
            history: vec![],
            forward_history: vec![],

            file_path: None,
            lang_id: None,
        }
    }

    pub fn with_lang(self, lang_id: LangId) -> Self {
        Self {
            lang_id: Some(lang_id),
            ..self
        }
    }

    pub fn with_text_from_rope(self, rope: Rope, lang_id: Option<LangId>) -> Self {
        let mut text = Text::default().with_rope(rope);

        if let Some(lang_id) = lang_id {
            if !text.parse(self.tree_sitter.clone(), lang_id) {
                error!("creation of parse_tuple failed");
            }
        }

        Self {
            text,
            ..self
        }
    }

    pub fn set_lang(&mut self, lang_id: Option<LangId>) {
        self.lang_id = lang_id;
    }

    fn clone_top(&mut self) {
        self.history.push(self.text.clone());
    }

    // fn after_change(&mut self) {
    //     // self.text.try_reparse_after_tree_update();
    //     self.forward_history.clear();
    // }

    pub fn prev(&mut self) -> bool {
        match self.history.pop() {
            None => false,
            Some(r) => {
                self.forward_history.push(self.text.clone());
                self.text = r;
                true
            }
        }
    }

    pub fn next(&mut self) -> bool {
        match self.forward_history.pop() {
            None => false,
            Some(r) => {
                self.history.push(self.text.clone());
                self.text = r;
                true
            }
        }
    }

    pub fn char_to_kind(&self, char_idx: usize) -> Option<&str> {
        // let byte_idx = self.text.rope.try_char_to_byte(char_idx).ok()?;
        // self.text.parsing.as_ref()?.tree.root_node().descendant_for_byte_range(byte_idx, byte_idx).map(|node| node.kind())
        None
    }
}

impl Buffer for BufferState {
    fn len_lines(&self) -> usize {
        self.text.rope.len_lines()
    }
    fn lines(&self) -> Box<dyn Iterator<Item=String> + '_> {
        // TODO this will fail for large files
        Box::new(self.text.rope.lines().map(|f| f.to_string()))
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn len_chars(&self) -> usize {
        self.text.rope.len_chars()
    }

    fn char_to_line(&self, char_idx: usize) -> Option<usize> {
        match self.text.rope.try_char_to_line(char_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    fn line_to_char(&self, line_idx: usize) -> Option<usize> {
        match self.text.rope.try_line_to_char(line_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    fn insert_char(&mut self, char_idx: usize, ch: char) -> bool {
        match self.text.rope.try_insert_char(char_idx, ch) {
            Ok(_) => {
                // TODO maybe this method should be moved to text object.
                let rope_clone = self.text.rope.clone();

                self.text.parsing.as_mut().map_or_else(
                    || {
                        error!("failed to acquire parse_tuple");
                    },
                    |r| {
                        r.update_parse_on_insert(&rope_clone, char_idx, char_idx + 1);
                    });

                true
            }
            Err(e) => {
                warn!("failed inserting char {} because {}", char_idx, e);
                false
            }
        }
    }

    fn insert_block(&mut self, char_idx: usize, block: &str) -> bool {
        // TODO maybe blocks will be more performant?
        let grapheme_len = block.graphemes(true).count();
        match self.text.rope.try_insert(char_idx, block) {
            Ok(_) => {
                let rope_clone = self.text.rope.clone();

                self.text.parsing.as_mut().map_or_else(
                    || {
                        error!("failed to acquire parse_tuple");
                    },
                    |r| {
                        r.update_parse_on_insert(&rope_clone, char_idx, char_idx + grapheme_len);
                    });

                true
            }
            Err(e) => {
                warn!("failed inserting block {} (len {}) because {}", char_idx, grapheme_len, e);
                false
            }
        }
    }

    fn remove(&mut self, char_idx_begin: usize, char_idx_end: usize) -> bool {
        if !(char_idx_end > char_idx_begin) {
            error!("requested removal of improper range ({}, {})", char_idx_begin, char_idx_end);
            return false;
        }

        match self.text.rope.try_remove(char_idx_begin..char_idx_end) {
            Ok(_) => {
                let rope_clone = self.text.rope.clone();

                self.text.parsing.as_mut().map_or_else(
                    || {
                        error!("failed to acquire parse_tuple");
                    },
                    |r| {
                        r.update_parse_on_delete(&rope_clone, char_idx_begin, char_idx_end);
                    });

                true
            }
            Err(e) => {
                warn!("failed removing char {:?}-{:?} because {}", char_idx_begin, char_idx_end, e);
                false
            }
        }
    }

    fn char_at(&self, char_idx: usize) -> Option<char> {
        self.text.rope.char_at(char_idx)
    }

    fn callback_for_parser<'a>(&'a self) -> Box<dyn FnMut(usize, Point) -> &'a [u8] + 'a> {
        self.text.rope.callback_for_parser()
    }
}