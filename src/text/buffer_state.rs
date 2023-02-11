use std::fmt::Debug;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use log::{debug, error, warn};
use ropey::iter::{Chars, Chunks};
use ropey::Rope;
use streaming_iterator::StreamingIterator;
use tree_sitter::Point;
use unicode_segmentation::UnicodeSegmentation;

use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::regex_search::{FindError, regex_find};
use crate::fs::path::SPath;
use crate::io::output::Output;
use crate::primitives::common_edit_msgs::{_apply_cem, CommonEditMsg};
use crate::primitives::cursor_set::{Cursor, CursorSet, Selection};
use crate::primitives::search_pattern::SearchPattern;
use crate::text::contents_and_cursors::ContentsAndCursors;
use crate::text::text_buffer::{LinesIter, TextBuffer};
use crate::tsw::lang_id::LangId;
use crate::tsw::parsing_tuple::ParsingTuple;
use crate::tsw::tree_sitter_wrapper::{HighlightItem, TreeSitterWrapper};
use crate::unpack_or;
use crate::w7e::navcomp_provider::StupidSubstituteMessage;

// TODO it would use a method "would_accept_cem" to be used in "on_input" but before "update"

/*
Ok, so I'd like to have multiple views of the same file. We can for a second even think that they
each have separate set of cursors. They definitely should share history of edits, at least until
"local edit history" is introduced ("undo and redo do not move the view"). Even when "local history"
is introduced, that still only means we have common history of edits with the view acting as filter
(or selector) of "which history elements are relevant". But this is far out, requires a lot of thinking.

Anyway, separate cursors but common history. That means that if cursor A < B edits in ViewA, cursor
B needs to be moved too. To avoid "communicating between views" it seams reasonable, to hold
*all views cursors inside single common BufferState*, and then just use the relevant one via
some kind of hash map. So I am NOT separating cursors from BufferState, even though they are view
specific. This is a good place to keep them all.
 */


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum BufferType {
    Full,
    SingleLine,
}

#[derive(Debug)]
pub struct BufferState {
    subtype: BufferType,

    tree_sitter_op: Option<Arc<TreeSitterWrapper>>,
    history: Vec<ContentsAndCursors>,
    history_pos: usize,

    lang_id: Option<LangId>,

    file: Option<SPath>,
}

impl BufferState {
    pub fn full(tree_sitter_op: Option<Arc<TreeSitterWrapper>>) -> BufferState {
        BufferState {
            subtype: BufferType::Full,
            tree_sitter_op,
            history: vec![ContentsAndCursors::default()],
            history_pos: 0,
            lang_id: None,
            file: None,
        }
    }

    pub fn simplified_single_line() -> BufferState {
        BufferState {
            subtype: BufferType::SingleLine,
            tree_sitter_op: None,
            history: vec![ContentsAndCursors::default()],
            history_pos: 0,
            lang_id: None,
            file: None,
        }
    }

    pub fn subtype(&self) -> &BufferType {
        &self.subtype
    }

    pub fn with_lang(self, lang_id: LangId) -> Self {
        if self.subtype != BufferType::Full {
            error!("setting lang in non TextBuffer::Full!");
        }

        Self {
            lang_id: Some(lang_id),
            ..self
        }
    }

    pub fn with_file_front(self, ff: SPath) -> Self {
        Self {
            file: Some(ff),
            ..self
        }
    }

    pub fn with_text<T: AsRef<str>>(self, text: T) -> Self {
        let rope = ropey::Rope::from_str(text.as_ref());
        Self {
            history: vec![ContentsAndCursors::default().with_rope(rope)],
            history_pos: 0,
            ..self
        }
    }

    /*
    This is expected to be used only in construction, it clears the history.
     */
    pub fn with_text_from_rope(self, rope: Rope, lang_id: Option<LangId>) -> Self {
        let copy_rope = rope.clone();
        let mut text = ContentsAndCursors::default().with_rope(rope);

        if let Some(lang_id) = lang_id {
            if let Some(tree_sitter) = self.tree_sitter_op.as_ref() {
                if text.parse(tree_sitter.clone(), lang_id) {
                    text.parsing.as_mut().map(|parsing| {
                        parsing.try_reparse(&copy_rope);
                    });
                } else {
                    error!("creation of parse_tuple failed");
                }
            } else {
                error!("will not parse, because TreeSitter not available - simplified buffer?");
            }
        }

        let res = Self {
            history: vec![text],
            history_pos: 0,
            ..self
        };

        res
    }

    pub fn char_range(&self, output: &mut dyn Output) -> Option<Range<usize>> {
        let rope = &self.text().rope;

        let sc = output.size_constraint();
        let visible_rect = unpack_or!(sc.visible_hint(), None);

        let first_line = visible_rect.upper_left().y as usize;
        let beyond_last_lane = visible_rect.lower_right().y as usize + 1;

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

    pub fn set_file_front(&mut self, ff_op: Option<SPath>) {
        // TODO on update, I should break the history
        self.file = ff_op;
    }

    pub fn get_path(&self) -> Option<&SPath> {
        self.file.as_ref()
    }

    pub fn set_lang(&mut self, lang_id: Option<LangId>) {
        if self.subtype != BufferType::Full {
            error!("setting lang in non TextBuffer::Full!");
        }

        self.lang_id = lang_id;
    }

    pub fn cursors(&self) -> &CursorSet {
        &self.text().cursor_set
    }

    pub fn apply_cem(&mut self,
                     mut cem: CommonEditMsg,
                     page_height: usize,
                     clipboard: Option<&ClipboardRef>,
    ) -> bool {
        if self.subtype == BufferType::SingleLine {
            if page_height != 1 {
                error!("page_height required to be 1 on SingleLine buffers!");
                return false;
            }

            match cem {
                CommonEditMsg::Char('\n') => {
                    error!("not adding newline to a single-line buffer");
                    return false;
                }
                CommonEditMsg::Block(block) => {
                    let new_block = block.replace("\n", "");
                    cem = CommonEditMsg::Block(new_block);
                }
                _ => {}
            }
        }

        let cem = cem;

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
        let (_diff_len_chars, any_change) = _apply_cem(cem.clone(), &mut cursors, self, page_height as usize, clipboard);

        //undo/redo invalidates cursors copy, so I need to watch for those
        match cem {
            CommonEditMsg::Undo | CommonEditMsg::Redo => {}
            _ => {
                self.text_mut().cursor_set = cursors;

                if !any_change {
                    self.undo_milestone();
                }
            }
        }

        any_change
    }

    /*
    This creates new milestone to undo/redo. The reason for it is that potentially multiple edits inform a single milestone.
    Returns false only if buffer have not changed since last milestone (TODO: that part is not implemented).

    set_milestone drops "forward history".
     */
    fn set_milestone(&mut self) -> bool {
        self.history.truncate(self.history_pos + 1);
        self.history.push(self.history[self.history_pos].clone());
        self.history_pos += 1;
        true
    }

    // to be used only in apply_cem
    fn undo_milestone(&mut self) {
        debug_assert!(self.history_pos + 1 == self.history.len());
        debug_assert!(self.history_pos > 0);
        self.history_pos -= 1;
        self.history.truncate(self.history_pos + 1);
    }

    // removes previous to last milestone, and moves last one to it's position.
    // used to chain multiple operations into a single milestone
    fn reduce_merge_milestone(&mut self) {
        debug_assert!(self.history_pos + 1 == self.history.len());
        debug_assert!(self.history_pos >= 1);

        self.history.remove(self.history_pos - 1);
        self.history_pos -= 1;
    }

    pub fn text(&self) -> &ContentsAndCursors {
        debug_assert!(self.history.len() >= self.history_pos);
        &self.history[self.history_pos]
    }

    pub fn text_mut(&mut self) -> &mut ContentsAndCursors {
        debug_assert!(self.history.len() >= self.history_pos);
        &mut self.history[self.history_pos]
    }

    /*
    This is an action destructive to cursor set - it uses only the supercursor.anchor as starting point for
    search.

    returns Ok(true) iff there was an occurrence
     */
    pub fn find_once(&mut self, pattern: &str) -> Result<bool, FindError> {
        self.set_milestone();

        match self.text_mut().find_once(pattern) {
            Err(e) => {
                // not even started the search: strip milestone and propagate error.
                self.undo_milestone();
                Err(e)
            }
            Ok(false) => {
                // there was no occurrences, so nothing changed - strip milestone.
                self.undo_milestone();
                Ok(false)
            }
            Ok(true) => {
                Ok(true)
            }
        }
    }

    pub fn streaming_iterator(&self) -> BufferStateStreamingIterator {
        BufferStateStreamingIterator {
            chunks: self.chunks(),
            curr_chunk: None,
        }
    }

    // returns whether a change happened. Undoes changes on fail.
    // TODO fuzzy that invariant: false => unchanged
    pub fn apply_stupid_substitute_messages(&mut self, stupid_messages: &Vec<StupidSubstituteMessage>, page_height: usize) -> bool {
        if stupid_messages.is_empty() {
            warn!("calling apply_stupid_substitute_messages with empty list");
            return false;
        }

        let mut res = false;

        for msg in stupid_messages.iter() {
            if self._apply_stupid_substitute_message(msg, page_height) {
                self.reduce_merge_milestone();
                res = true;
            }
        }

        res
    }


    // returns whether a change happened. Undoes changes on fail.
    // TODO fuzzy that invariant: false => unchanged
    // TODO maybe, just maybe, these stupid messages should go to CEM, not sure. Because moving them out already made me forgot about updating navcomp and updating treesitter.
    fn _apply_stupid_substitute_message(&mut self,
                                        stupid_message: &StupidSubstituteMessage,
                                        page_height: usize,
    ) -> bool {
        if !self.text().cursor_set.are_simple() {
            error!("refuse to apply stupid_edit_to_cem: cursors are not simple");
            return false;
        }

        let begin = unpack_or!(stupid_message.stupid_range.0.to_real_cursor(self), false, "failed to cast (1) to real cursor");
        let end = unpack_or!(stupid_message.stupid_range.1.to_real_cursor(self), false, "failed to cast (2) to real cursor");

        self.set_milestone();

        // removing old item
        if stupid_message.stupid_range.0 != stupid_message.stupid_range.1 {
            if self.apply_cem(
                CommonEditMsg::DeleteBlock { char_range: begin.a..end.a },
                page_height,
                None,
            ) {
                let rope = self.text().rope.clone(); // shallow copy
                self.text_mut().parsing.as_mut().map_or_else(
                    || {
                        debug!("failed to acquire parse_tuple 1");
                    },
                    |r| {
                        r.update_parse_on_delete(&rope, begin.a, end.a);
                    });
            } else {
                error!("failed to remove range [{}..{}) from rope", begin.a, end.a);
                self.undo_milestone();
                return false;
            }
        }

        if !stupid_message.substitute.is_empty() {
            let what = stupid_message.substitute.clone();
            let char_len = what.graphemes(true).count();
            if self.apply_cem(
                // TODO unnecessary clone
                CommonEditMsg::InsertBlock { char_pos: begin.a, what },
                page_height,
                None,
            ) {
                let rope = self.text().rope.clone(); // shallow copy
                self.text_mut().parsing.as_mut().map_or_else(
                    || {
                        debug!("failed to acquire parse_tuple 1");
                    },
                    |r| {
                        r.update_parse_on_insert(&rope, begin.a, begin.a + char_len);
                    });
            } else {
                error!("failed to remove range [{}..{}) from rope", begin.a, end.a);
                self.undo_milestone();
                return false;
            }
        }

        true
    }
}

impl ToString for BufferState {
    fn to_string(&self) -> String {
        let mut output = String::new();

        let mut line_it = self.lines();
        while let Some(line) = line_it.next() {
            output += line.as_str()
        }

        output
    }
}

impl TextBuffer for BufferState {
    fn len_lines(&self) -> usize {
        self.text().rope.len_lines()
    }

    fn lines(&self) -> LinesIter {
        LinesIter::new(self.text().rope.chars())
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn len_chars(&self) -> usize {
        self.text().rope.len_chars()
    }

    fn len_bytes(&self) -> usize {
        self.text().rope.len_bytes()
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

    fn byte_to_char(&self, byte_idx: usize) -> Option<usize> {
        self.text().rope.try_byte_to_char(byte_idx).ok()
    }

    fn char_to_byte(&self, char_idx: usize) -> Option<usize> {
        self.text().rope.try_char_to_byte(char_idx).ok()
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

    fn chunks(&self) -> Chunks {
        self.text().rope.chunks()
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

pub struct BufferStateStreamingIterator<'a> {
    chunks: Chunks<'a>,
    curr_chunk: Option<&'a str>,
}

impl<'a> StreamingIterator for BufferStateStreamingIterator<'a> {
    type Item = [u8];

    fn advance(&mut self) {
        self.curr_chunk = self.chunks.next();
    }

    fn get(&self) -> Option<&Self::Item> {
        self.curr_chunk.map(|c| c.as_bytes())
    }
}