use std::fmt::{Debug};
use std::ops::Range;
use std::rc::Rc;
use std::string::String;

use log::{error, warn};
use ropey::Rope;
use tree_sitter::{Point};
use unicode_segmentation::UnicodeSegmentation;
use crate::fs::file_front::FileFront;
use crate::fs::filesystem_front::SomethingToSave;
use crate::Output;

use crate::text::buffer::Buffer;
use crate::tsw::lang_id::LangId;
use crate::tsw::parsing_tuple::ParsingTuple;
use crate::tsw::tree_sitter_wrapper::{HighlightItem, TreeSitterWrapper};

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
        if let Some(parsing_tuple) = tree_sitter.new_parse(lang_id) {
            self.parsing = Some(parsing_tuple);

            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct BufferState {
    tree_sitter: Rc<TreeSitterWrapper>,

    text: Text,

    history: Vec<Text>,
    forward_history: Vec<Text>,

    lang_id: Option<LangId>,

    file: Option<FileFront>,
}

impl BufferState {
    pub fn new(tree_sitter: Rc<TreeSitterWrapper>) -> BufferState {
        BufferState {
            tree_sitter,
            text: Text::default(),
            history: vec![],
            forward_history: vec![],

            file: None,
            lang_id: None,
        }
    }

    pub fn char_range(&self, output: &mut dyn Output) -> Option<Range<usize>> {
        let rope = &self.text.rope;

        let first_line = output.size_constraint().hint().upper_left().y as usize;
        let beyond_last_lane = output.size_constraint().hint().lower_right().y as usize + 1;

        let first_char_idx = rope.try_line_to_char(first_line).ok()?;
        let beyond_last_char_idx = rope.try_line_to_char(beyond_last_lane).ok()?;

        Some(first_char_idx..beyond_last_char_idx)
    }

    pub fn highlight(&self, char_range_op: Option<Range<usize>>) -> Vec<HighlightItem> {
        self.text.parsing.as_ref().map(|parsing| {
            parsing.highlight_iter(&self.text.rope, char_range_op)
        }).flatten().unwrap_or(vec![])
    }

    pub fn with_lang(self, lang_id: LangId) -> Self {
        Self {
            lang_id: Some(lang_id),
            ..self
        }
    }

    pub fn set_file_front(&mut self, ff_op: Option<FileFront>) {
        self.file = ff_op;
    }

    pub fn get_file_front(&self) -> Option<&FileFront> {
        self.file.as_ref()
    }

    pub fn with_file_front(self, ff: FileFront) -> Self {
        Self {
            file: Some(ff),
            ..self
        }
    }

    pub fn with_text_from_rope(self, rope: Rope, lang_id: Option<LangId>) -> Self {
        let copy_rope = rope.clone();
        let mut text = Text::default().with_rope(rope);

        if let Some(lang_id) = lang_id {
            if text.parse(self.tree_sitter.clone(), lang_id) {
                text.parsing.as_mut().map(|parsing| {
                    parsing.try_reparse(&copy_rope);
                });
            } else {
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
}

impl Buffer for BufferState {
    fn len_lines(&self) -> usize {
        self.text.rope.len_lines()
    }
    fn lines(&self) -> Box<dyn Iterator<Item=String> + '_> {
        // TODO this will fail for large files
        // TODO Hmm, I am also not sure what will happen if a line is between two slices.
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
                        error!("failed to acquire parse_tuple 1");
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
                        error!("failed to acquire parse_tuple 2");
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
                        error!("failed to acquire parse_tuple 3");
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

impl SomethingToSave for BufferState {
    fn get_slices(&self) -> Box<dyn Iterator<Item=&[u8]> + '_> {
        Box::new(self.text.rope.chunks().map(|chunk| chunk.as_bytes()))
    }
}