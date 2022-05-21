use std::fmt::{Debug};
use std::ops::Range;
use std::rc::Rc;
use std::string::String;
use std::time::SystemTime;

use log::{debug, error, warn};
use ropey::iter::Chars;
use ropey::Rope;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Point};
use unicode_segmentation::UnicodeSegmentation;
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::file_front::FileFront;
use crate::fs::filesystem_front::SomethingToSave;
use crate::Output;
use crate::primitives::common_edit_msgs::{apply_cem, CommonEditMsg};
use crate::primitives::cursor_set::CursorSet;

use crate::text::buffer::{Buffer, LinesIter};
use crate::text::diff_oracle::DiffOracle;
use crate::tsw::lang_id::LangId;
use crate::tsw::parsing_tuple::ParsingTuple;
use crate::tsw::tree_sitter_wrapper::{HighlightItem, TreeSitterWrapper};

#[derive(Clone, Debug, Default)]
pub struct Text {
    pub rope: Rope,
    pub parsing: Option<ParsingTuple>,
    pub cursor_set: CursorSet,
}


impl Text {
    pub fn with_rope(self, rope: Rope) -> Self {
        Self {
            rope,
            ..self
        }
    }

    pub fn with_cursor_set(self, cursor_set: CursorSet) -> Self {
        Self {
            cursor_set,
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

#[derive(Debug)]
pub struct BufferState {
    tree_sitter: Rc<TreeSitterWrapper>,

    history: Vec<Text>,
    history_pos: usize,

    lang_id: Option<LangId>,

    file: Option<FileFront>,
}

impl BufferState {
    pub fn new(tree_sitter: Rc<TreeSitterWrapper>) -> BufferState {
        let mut res = BufferState {
            tree_sitter,
            history: vec![Text::default()],
            history_pos: 0,

            file: None,
            lang_id: None,
        };

        res
    }

    pub fn with_lang(self, lang_id: LangId) -> Self {
        Self {
            lang_id: Some(lang_id),
            ..self
        }
    }

    pub fn with_file_front(self, ff: FileFront) -> Self {
        Self {
            file: Some(ff),
            ..self
        }
    }

    /*
    This is expected to be used only in construction, it clears the history.
     */
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

        let mut res = Self {
            history: vec![text],
            history_pos: 0,
            ..self
        };

        res
    }

    pub fn char_range(&self, output: &mut dyn Output) -> Option<Range<usize>> {
        let rope = &self.text().rope;

        let first_line = output.size_constraint().visible_hint().upper_left().y as usize;
        let beyond_last_lane = output.size_constraint().visible_hint().lower_right().y as usize + 1;

        let first_char_idx = rope.try_line_to_char(first_line).ok()?;
        let beyond_last_char_idx = rope.try_line_to_char(beyond_last_lane).ok()?;

        Some(first_char_idx..beyond_last_char_idx)
    }

    // TODO move to text?
    pub fn highlight(&self, char_range_op: Option<Range<usize>>) -> Vec<HighlightItem> {
        let text = self.text();
        text.parsing.as_ref().map(|parsing| {
            parsing.highlight_iter(&text.rope, char_range_op)
        }).flatten().unwrap_or(vec![])
    }

    pub fn set_file_front(&mut self, ff_op: Option<FileFront>) {
        self.file = ff_op;
    }

    pub fn get_file_front(&self) -> Option<&FileFront> {
        self.file.as_ref()
    }

    pub fn set_lang(&mut self, lang_id: Option<LangId>) {
        self.lang_id = lang_id;
    }

    pub fn apply_cem(&mut self, cem: CommonEditMsg, page_height: usize, clipboard: Option<&ClipboardRef>) -> bool {
        /*
        TODO the fact that Undo/Redo requires special handling here a lot suggests that maybe these shouldn't be CEMs. But it works now.
         */
        match cem {
            CommonEditMsg::Undo | CommonEditMsg::Redo => {}
            _ => {
                self.set_milestone();
            }
        }

        // TODO optimise
        let mut cursors = self.text().cursor_set.clone();
        let (diff_len_chars, any_change) = apply_cem(cem.clone(), &mut cursors, self, page_height as usize, clipboard);

        //undo/redo invalidates cursors copy, so I need to watch for those

        match cem {
            CommonEditMsg::Undo | CommonEditMsg::Redo => {}
            _ => {
                self.text_mut().cursor_set = cursors;

                if !any_change {
                    self.strip_milestone();
                }
            }
        }

        any_change
    }

    /*
    This creates new milestone to undo/redo. The reason for it is that potentially multiple edits inform a single milestone.
    Returns false only if buffer have not changed since last milestone.

    set_milestone implies and executes drop_forward_history
     */
    fn set_milestone(&mut self) -> bool {
        self.history.truncate(self.history_pos + 1);
        self.history.push(self.history[self.history_pos].clone());
        self.history_pos += 1;
        true
    }

    // to be used only in apply_cem
    fn strip_milestone(&mut self) {
        debug_assert!(self.history_pos + 1 == self.history.len());
        debug_assert!(self.history_pos > 0);
        self.history_pos -= 1;
        self.history.truncate(self.history_pos + 1);
    }

    pub fn text(&self) -> &Text {
        debug_assert!(self.history.len() >= self.history_pos);
        &self.history[self.history_pos]
    }

    pub fn text_mut(&mut self) -> &mut Text {
        debug_assert!(self.history.len() >= self.history_pos);
        &mut self.history[self.history_pos]
    }
}

impl Buffer for BufferState {
    fn len_lines(&self) -> usize {
        self.text().rope.len_lines()
    }

    fn new_lines(&self) -> LinesIter {
        self.text().rope.new_lines()
    }


    fn is_editable(&self) -> bool {
        true
    }

    fn len_chars(&self) -> usize {
        self.text().rope.len_chars()
    }

    fn char_to_line(&self, char_idx: usize) -> Option<usize> {
        match self.text().rope.try_char_to_line(char_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    fn line_to_char(&self, line_idx: usize) -> Option<usize> {
        match self.text().rope.try_line_to_char(line_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    fn insert_char(&mut self, char_idx: usize, ch: char) -> bool {
        let text = self.text_mut();
        match text.rope.try_insert_char(char_idx, ch) {
            Ok(_) => {
                // TODO maybe this method should be moved to text object.
                let rope_clone = text.rope.clone();

                text.parsing.as_mut().map_or_else(
                    || {
                        debug!("failed to acquire parse_tuple 1");
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
        let text = self.text_mut();

        match text.rope.try_insert(char_idx, block) {
            Ok(_) => {
                let rope_clone = text.rope.clone();

                text.parsing.as_mut().map_or_else(
                    || {
                        debug!("failed to acquire parse_tuple 2");
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

        let text = self.text_mut();
        match text.rope.try_remove(char_idx_begin..char_idx_end) {
            Ok(_) => {
                let rope_clone = text.rope.clone();

                text.parsing.as_mut().map_or_else(
                    || {
                        debug!("failed to acquire parse_tuple 3");
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
        self.text().rope.char_at(char_idx)
    }

    fn chars(&self) -> Chars {
        self.text().rope.chars()
    }

    fn callback_for_parser<'a>(&'a self) -> Box<dyn FnMut(usize, Point) -> &'a [u8] + 'a> {
        self.text().rope.callback_for_parser()
    }

    fn can_undo(&self) -> bool {
        self.history_pos > 0
    }

    fn can_redo(&self) -> bool {
        self.history_pos + 1 < self.history.len()
    }

    fn undo(&mut self) -> bool {
        debug!("UNDO pos {} len {}", self.history_pos, self.history.len());
        if self.history_pos > 0 {
            self.history_pos -= 1;
            true
        } else { false }
    }

    fn redo(&mut self) -> bool {
        debug!("REDO pos {} len {}", self.history_pos, self.history.len());
        if self.history_pos + 1 < self.history.len() {
            self.history_pos += 1;
            true
        } else { false }
    }
}

impl SomethingToSave for BufferState {
    fn get_slices(&self) -> Box<dyn Iterator<Item=&[u8]> + '_> {
        Box::new(self.text().rope.chunks().map(|chunk| chunk.as_bytes()))
    }
}