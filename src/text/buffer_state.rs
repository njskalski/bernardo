use std::cmp::max;
use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;

use log::{debug, error, info, warn};
use ropey::iter::{Chars, Chunks};
use ropey::Rope;
use streaming_iterator::StreamingIterator;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::cursor::cursor_set::CursorSet;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::filename_to_language::filename_to_language;
use crate::experiments::regex_search::FindError;
use crate::fs::path::SPath;
use crate::io::output::Output;
use crate::primitives::common_edit_msgs::{apply_common_edit_message, CommonEditMsg};
use crate::primitives::has_invariant::HasInvariant;
use crate::primitives::xy::XY;
use crate::text::contents_and_cursors::ContentsAndCursors;
use crate::text::text_buffer::{LinesIter, TextBuffer};
use crate::tsw::lang_id::LangId;
use crate::tsw::tree_sitter_wrapper::{HighlightItem, TreeSitterWrapper};
use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
use crate::w7e::navcomp_provider::StupidSubstituteMessage;
use crate::widget::widget::WID;
use crate::widgets::main_view::main_view::DocumentIdentifier;
use crate::{unpack_or, unpack_or_e};

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

pub struct SetFilePathResult {
    pub document_id: DocumentIdentifier,
    pub path_changed: bool,
}

#[derive(Debug)]
pub struct BufferState {
    subtype: BufferType,

    tree_sitter_op: Option<Arc<TreeSitterWrapper>>,
    history: Vec<ContentsAndCursors>,
    history_pos: usize,
    last_save_pos: Option<usize>,

    lang_id: Option<LangId>,

    document_identifier: DocumentIdentifier,
}

impl BufferState {
    pub fn into_bsr(self) -> BufferSharedRef {
        BufferSharedRef::new_from_buffer(self)
    }

    pub fn apply_common_edit_message(
        &mut self,
        mut cem: CommonEditMsg,
        widget_id: WID,
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
                    // TODO not sure if this should be here
                    let new_block = block.replace("\n", "");
                    cem = CommonEditMsg::Block(new_block);
                }
                _ => {}
            }
        }

        let cem = cem;
        let mut cursors_copy = unpack_or_e!(self.text().get_cursor_set(widget_id), false, "cursor set not found").clone();

        /*
        TODO the fact that Undo/Redo requires special handling here a lot suggests that maybe these shouldn't be CEMs. But it works now.
         */

        let set_milestone = match cem {
            CommonEditMsg::Undo | CommonEditMsg::Redo => false,
            _ => self.set_milestone(),
        };

        let any_change = apply_common_edit_message(cem.clone(), &mut cursors_copy, &mut vec![], self, page_height as usize, clipboard);

        //undo/redo invalidates cursors copy, so I need to watch for those
        match cem {
            CommonEditMsg::Undo | CommonEditMsg::Redo => {}
            _ => {
                self.text_mut().set_cursor_set(widget_id, cursors_copy);
                if !any_change && set_milestone {
                    self.undo_milestone();
                }
            }
        }

        any_change
    }

    /*
     Returns whether a change happened. Undoes changes on fail.
     Used in "reformat".
    */
    // TODO fuzzy that invariant: false => unchanged
    pub fn apply_stupid_substitute_messages(
        &mut self,
        /*
        This is not necessary, but I put it so I don't have to think about reducing it now.
         */
        widget_id: WID,
        stupid_messages: &Vec<StupidSubstituteMessage>,
        page_height: usize,
    ) -> bool {
        if stupid_messages.is_empty() {
            warn!("calling apply_stupid_substitute_messages with empty list");
            return false;
        }

        let mut res = false;

        for msg in stupid_messages.iter() {
            if self._apply_stupid_substitute_message(widget_id, msg, page_height) {
                self.reduce_merge_milestone();
                res = true;
            }
        }

        debug_assert!(self.check_invariant());

        res
    }

    /*
     Returns whether a change happened. Undoes changes on fail.
     Used in "reformat".
    */
    // TODO fuzzy that invariant: false => unchanged
    // TODO maybe, just maybe, these stupid messages should go to CEM, not sure. Because moving them out already made me forgot about updating navcomp and updating treesitter.
    fn _apply_stupid_substitute_message(
        &mut self,
        /*
        This is not necessary, but I put it so I don't have to think about reducing it now.
         */
        widget_id: WID,
        stupid_message: &StupidSubstituteMessage,
        page_height: usize,
    ) -> bool {
        {
            let cursor_set = unpack_or!(
                self.text().get_cursor_set(widget_id),
                false,
                "failed _apply_stupid_substitute_message - WID not found"
            );
            if cursor_set.are_simple() {
                error!("refuse to apply stupid_edit_to_cem: cursors are not simple");
                return false;
            }
        }

        let begin = unpack_or!(
            stupid_message.stupid_range.0.to_real_cursor(self),
            false,
            "failed to cast (1) to real cursor"
        );
        let end = unpack_or!(
            stupid_message.stupid_range.1.to_real_cursor(self),
            false,
            "failed to cast (2) to real cursor"
        );

        let set_milestone = self.set_milestone();

        // removing old item
        if stupid_message.stupid_range.0 != stupid_message.stupid_range.1 {
            if self.apply_common_edit_message(
                CommonEditMsg::DeleteBlock {
                    char_range: begin.a..end.a,
                },
                widget_id,
                page_height,
                None,
            ) {
                let rope = self.text().rope().clone(); // shallow copy
                self.text_mut().parsing_mut().map_or_else(
                    || {
                        debug!("failed to acquire parse_tuple 1");
                    },
                    |r| {
                        r.update_parse_on_delete(&rope, begin.a, end.a);
                    },
                );
            } else {
                error!("failed to remove range [{}..{}) from rope", begin.a, end.a);
                if set_milestone {
                    self.undo_milestone();
                }
                return false;
            }
        }

        if !stupid_message.substitute.is_empty() {
            let what = stupid_message.substitute.clone();
            let char_len = what.graphemes(true).count();
            if self.apply_common_edit_message(
                // TODO unnecessary clone
                CommonEditMsg::InsertBlock { char_pos: begin.a, what },
                widget_id,
                page_height,
                None,
            ) {
                let rope = self.text().rope().clone(); // shallow copy
                self.text_mut().parsing_mut().map_or_else(
                    || {
                        debug!("failed to acquire parse_tuple 1");
                    },
                    |r| {
                        r.update_parse_on_insert(&rope, begin.a, begin.a + char_len);
                    },
                );
            } else {
                error!("failed to remove range [{}..{}) from rope", begin.a, end.a);
                if set_milestone {
                    self.undo_milestone();
                }
                return false;
            }
        }

        true
    }

    pub fn get_visible_chars_range(&self, output: &mut dyn Output) -> Option<Range<usize>> {
        let rope = self.text().rope();

        let visible_rect = output.visible_rect();

        let first_line = visible_rect.upper_left().y as usize;
        let beyond_last_lane = visible_rect.lower_right().y as usize + 1;

        let first_char_idx = rope.try_line_to_char(first_line).ok()?;
        let beyond_last_char_idx = rope.try_line_to_char(beyond_last_lane).unwrap_or(rope.len_chars()); // if you do add +1 here, treesitter fails.

        Some(first_char_idx..beyond_last_char_idx)
    }

    // WidgetId should correspond to EditorWidget and NOT EditorView.
    pub fn cursors(&self, widget_id: WID) -> Option<&CursorSet> {
        self.text().get_cursor_set(widget_id)
    }

    // WidgetId should correspond to EditorWidget and NOT EditorView.
    pub fn cursors_mut(&mut self, widget_id: WID) -> Option<&mut CursorSet> {
        self.text_mut().get_cursor_set_mut(widget_id)
    }

    /*
    This is an action destructive to cursor set - it uses only the supercursor.anchor as starting point for
    search.

    returns Ok(true) iff there was an occurrence

    // TODO change pattern from str to enum we created
     */
    pub fn find_once(&mut self, widget_id: WID, pattern: &str) -> Result<bool, FindError> {
        let set_milestone = self.set_milestone();

        let result = match self.text_mut().find_once(widget_id, pattern) {
            Err(e) => {
                // not even started the search: strip milestone and propagate error.
                if set_milestone {
                    self.undo_milestone();
                }
                Err(e)
            }
            Ok(false) => {
                // there was no occurrences, so nothing changed - strip milestone.
                if set_milestone {
                    self.undo_milestone();
                }
                Ok(false)
            }
            Ok(true) => Ok(true),
        };

        debug_assert!(self.check_invariant());

        result
    }

    pub fn full(tree_sitter_op: Option<Arc<TreeSitterWrapper>>, document_identifier: DocumentIdentifier) -> BufferState {
        let res = BufferState {
            subtype: BufferType::Full,
            tree_sitter_op,
            history: vec![ContentsAndCursors::empty()],
            history_pos: 0,
            last_save_pos: None,
            lang_id: None,
            document_identifier,
        };

        debug_assert!(res.check_invariant());

        res
    }

    pub fn get_path(&self) -> Option<&SPath> {
        self.document_identifier.file_path.as_ref()
    }

    pub fn get_document_identifier(&self) -> &DocumentIdentifier {
        &self.document_identifier
    }

    pub fn get_lang_id(&self) -> Option<LangId> {
        self.lang_id.clone()
    }

    // TODO move to text?
    pub fn highlight(&self, char_range_op: Option<Range<usize>>) -> Vec<HighlightItem> {
        let text = self.text();
        text.parsing()
            .map(|parsing| parsing.highlight_iter(text.rope(), char_range_op))
            .flatten()
            .unwrap_or(vec![])
    }

    // TODO merge with above?
    pub fn smallest_highlight(&self, char_idx: usize) -> Option<HighlightItem> {
        let text = self.text();
        if text.rope().len_chars() < char_idx + 1 {
            return None;
        };

        let parsing = text.parsing()?;
        let items = parsing.highlight_iter(text.rope(), Some(char_idx..char_idx + 1))?;

        let mut best: Option<HighlightItem> = None;

        for item in items.into_iter() {
            if item.char_begin <= char_idx && char_idx <= item.char_end {
                if let Some(old_best) = best.as_ref() {
                    let old_len = old_best.char_end - old_best.char_begin;
                    let new_len = item.char_end - item.char_begin;
                    if old_len > new_len {
                        best = Some(item);
                    }
                } else {
                    best = Some(item);
                }
            }
        }

        best
    }

    pub fn remove_history(&mut self) {
        let is_saved = self.is_saved();

        if self.history_pos != 0 {
            self.history.swap(0, self.history_pos)
        }
        self.history.truncate(1);
        self.history_pos = 0;

        if is_saved {
            self.last_save_pos = Some(0);
        } else {
            self.last_save_pos = None;
        }

        debug_assert!(self.check_invariant());
    }

    /* removes previous to last milestone, and moves last one to it's position.
      used to chain multiple operations into a single milestone
    */
    fn reduce_merge_milestone(&mut self) {
        debug_assert!(self.history_pos + 1 == self.history.len());
        debug_assert!(self.history_pos >= 1);

        self.history.remove(self.history_pos - 1);
        self.history_pos -= 1;

        if let Some(last_save_pos) = self.last_save_pos {
            if last_save_pos == self.history_pos + 1 {
                self.last_save_pos = Some(last_save_pos - 1);
            } else if last_save_pos >= self.history.len() {
                self.last_save_pos = None;
            }
        }

        debug_assert!(self.check_invariant());
    }

    // TODO overload and optimise?
    pub fn size(&self) -> XY {
        let mut size = XY::ZERO;

        size.y = self.len_lines() as u16; // TODO overflow

        let mut lines_iter = self.lines();
        while let Some(line) = lines_iter.next() {
            size.x = max(size.x, line.width() as u16) // TODO overflow
        }

        size
    }

    /*
    Returns updated DocumentIdentifier
     */
    pub fn set_file_path(&mut self, file_path_op: Option<SPath>) -> SetFilePathResult {
        // TODO on update, I should break the history

        if file_path_op.is_none() {
            warn!("I can't think about scenario where we change ff to None, but here it happened");
        }

        let changed = self.document_identifier.file_path != file_path_op;

        self.document_identifier.file_path = file_path_op;

        debug_assert!(self.check_invariant());

        SetFilePathResult {
            document_id: self.document_identifier.clone(),
            path_changed: changed,
        }
    }

    pub fn set_lang(&mut self, lang_id: Option<LangId>) {
        if self.subtype != BufferType::Full {
            error!("setting lang in non TextBuffer::Full!");
        }

        self.lang_id = lang_id;
        self.set_parsing_tuple();

        debug_assert!(self.check_invariant());
    }

    /*
    This creates new milestone to undo/redo. The reason for it is that potentially multiple edits inform a single milestone.
    Returns false only if buffer have not changed since last milestone.

    set_milestone drops "forward history".
     */
    fn set_milestone(&mut self) -> bool {
        if self.is_saved() {
            return false;
        }

        self.history.truncate(self.history_pos + 1);
        self.history.push(self.history[self.history_pos].clone());
        self.history_pos += 1;

        debug_assert!(self.check_invariant());
        true
    }

    fn set_parsing_tuple(&mut self) -> bool {
        let lang_id = match self.lang_id {
            Some(li) => li,
            None => match self.get_path().map(filename_to_language).flatten() {
                None => {
                    info!("couldn't determine language: path = {:?}", self.get_path());
                    return false;
                }
                Some(lang_id) => lang_id,
            },
        };

        let copy_rope = self.text().rope().clone();

        let result = if let Some(tree_sitter_clone) = self.tree_sitter_op.as_ref().map(|r| r.clone()) {
            let parse_success: bool = self.text_mut().parse(tree_sitter_clone, lang_id);

            // TODO I honestly don't remember why I reparse here
            if parse_success {
                self.text_mut().parsing_mut().map(|parsing| {
                    if !parsing.try_reparse(&copy_rope) {
                        error!("failed try_reparse");
                    }
                });
                true
            } else {
                error!("creation of parse_tuple failed");
                false
            }
        } else {
            error!("will not parse, because TreeSitter not available - simplified buffer?");
            false
        };

        debug_assert!(self.check_invariant());

        result
    }

    pub fn simplified_single_line() -> BufferState {
        let doc_id = DocumentIdentifier::new_unique();
        let res = BufferState {
            subtype: BufferType::SingleLine,
            tree_sitter_op: None,
            history: vec![ContentsAndCursors::empty()],
            history_pos: 0,
            last_save_pos: None,
            lang_id: None,
            document_identifier: doc_id,
        };

        debug_assert!(res.check_invariant());

        res
    }

    pub fn streaming_iterator(&self) -> BufferStateStreamingIterator {
        BufferStateStreamingIterator {
            chunks: self.chunks(),
            curr_chunk: None,
        }
    }

    pub fn subtype(&self) -> &BufferType {
        &self.subtype
    }

    pub fn initialize_for_widget(&mut self, widget_id: WID, cursors_op: Option<CursorSet>) -> bool {
        let cursor_set = cursors_op.unwrap_or(CursorSet::single());
        let result = self.history[self.history_pos].add_cursor_set(widget_id, cursor_set);

        debug_assert!(self.check_invariant());

        result
    }

    pub fn text(&self) -> &ContentsAndCursors {
        debug_assert!(self.check_invariant());

        &self.history[self.history_pos]
    }

    pub fn text_mut(&mut self) -> &mut ContentsAndCursors {
        debug_assert!(self.check_invariant());

        &mut self.history[self.history_pos]
    }

    // to be used only in apply_cem
    fn undo_milestone(&mut self) {
        debug_assert!(self.history_pos + 1 == self.history.len());
        debug_assert!(self.history_pos > 0);
        self.history_pos -= 1;
        self.history.truncate(self.history_pos + 1);

        if let Some(last_save_pos) = self.last_save_pos {
            if last_save_pos >= self.history.len() {
                self.last_save_pos = None;
            }
        }

        debug_assert!(self.check_invariant());
    }

    pub fn with_lang(self, lang_id: LangId) -> Self {
        if self.subtype != BufferType::Full {
            error!("setting lang in non TextBuffer::Full!");
        }

        let res = Self {
            lang_id: Some(lang_id),
            ..self
        };

        debug_assert!(res.check_invariant());

        res
    }

    pub fn with_text<T: AsRef<str>>(self, text: T) -> Self {
        let rope = ropey::Rope::from_str(text.as_ref());

        let mut result = Self {
            history: vec![ContentsAndCursors::empty().with_rope(rope)],
            history_pos: 0,
            ..self
        };

        result.set_parsing_tuple();

        debug_assert!(result.check_invariant());

        result
    }

    pub fn with_maked_as_saved(self) -> Self {
        let pos = self.history_pos;

        Self {
            last_save_pos: Some(pos),
            ..self
        }
    }

    /*
    Destroys history
     */
    pub fn set_text<T: AsRef<str>>(&mut self, text: T) {
        self.history = vec![ContentsAndCursors::empty().with_rope(Rope::from_str(text.as_ref()))];
        self.history_pos = 0;

        self.set_parsing_tuple();
        self.check_invariant();
    }

    /*
    This is expected to be used only in construction, it clears the history.
     */
    pub fn with_text_from_rope(self, rope: Rope, lang_id: Option<LangId>) -> Self {
        let text = ContentsAndCursors::empty().with_rope(rope);

        let mut res = Self {
            history: vec![text],
            history_pos: 0,
            lang_id,
            ..self
        };

        res.set_parsing_tuple();

        debug_assert!(res.check_invariant());

        res
    }

    pub fn can_redo(&self) -> bool {
        self.history_pos + 1 < self.history.len()
    }

    pub fn can_undo(&self) -> bool {
        self.history_pos > 0
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
    fn byte_to_char(&self, byte_idx: usize) -> Option<usize> {
        self.text().rope().try_byte_to_char(byte_idx).ok()
    }

    fn char_at(&self, char_idx: usize) -> Option<char> {
        self.text().rope().get_chars_at(char_idx).map(|mut chars| chars.next()).flatten()
    }

    fn char_to_byte(&self, char_idx: usize) -> Option<usize> {
        self.text().rope().try_char_to_byte(char_idx).ok()
    }

    fn char_to_line(&self, char_idx: usize) -> Option<usize> {
        match self.text().rope().try_char_to_line(char_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    fn chars(&self) -> Chars {
        self.text().rope().chars()
    }

    fn chunks(&self) -> Chunks {
        self.text().rope().chunks()
    }

    fn insert_block(&mut self, char_idx: usize, block: &str) -> bool {
        // TODO maybe blocks will be more performant?
        let grapheme_len = block.graphemes(true).count();
        let text = self.text_mut();

        let result = match text.rope_mut().try_insert(char_idx, block) {
            Ok(_) => {
                let rope_clone = text.rope().clone();

                text.parsing_mut().map_or_else(
                    || {
                        debug!("failed to acquire parse_tuple 2");
                    },
                    |r| {
                        r.update_parse_on_insert(&rope_clone, char_idx, char_idx + grapheme_len);
                    },
                );

                true
            }
            Err(e) => {
                warn!("failed inserting block {} (len {}) because {}", char_idx, grapheme_len, e);
                false
            }
        };

        debug_assert!(self.check_invariant());

        result
    }

    fn insert_char(&mut self, char_idx: usize, ch: char) -> bool {
        let text = self.text_mut();
        let result = match text.rope_mut().try_insert_char(char_idx, ch) {
            Ok(_) => {
                // TODO maybe this method should be moved to text object.
                let rope_clone = text.rope().clone();

                text.parsing_mut().map_or_else(
                    || {
                        debug!("failed to acquire parse_tuple 1");
                    },
                    |r| {
                        r.update_parse_on_insert(&rope_clone, char_idx, char_idx + 1);
                    },
                );

                true
            }
            Err(e) => {
                warn!("failed inserting char {} because {}", char_idx, e);
                false
            }
        };

        debug_assert!(self.check_invariant());

        result
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn len_bytes(&self) -> usize {
        self.text().rope().len_bytes()
    }

    fn len_chars(&self) -> usize {
        self.text().rope().len_chars()
    }

    fn len_lines(&self) -> usize {
        self.text().rope().len_lines()
    }

    fn lines(&self) -> LinesIter {
        LinesIter::new(self.text().rope().chars())
    }

    fn line_to_char(&self, line_idx: usize) -> Option<usize> {
        match self.text().rope().try_line_to_char(line_idx) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    fn redo(&mut self) -> bool {
        debug!("REDO pos {} len {}", self.history_pos, self.history.len());
        let result = if self.history_pos + 1 < self.history.len() {
            self.history_pos += 1;
            true
        } else {
            false
        };

        debug_assert!(self.check_invariant());

        result
    }

    fn remove(&mut self, char_idx_begin: usize, char_idx_end: usize) -> bool {
        if !(char_idx_end > char_idx_begin) {
            error!("requested removal of improper range ({}, {})", char_idx_begin, char_idx_end);
            return false;
        }

        let text = self.text_mut();
        let result = match text.rope_mut().try_remove(char_idx_begin..char_idx_end) {
            Ok(_) => {
                let rope_clone = text.rope().clone();

                text.parsing_mut().map_or_else(
                    || {
                        debug!("failed to acquire parse_tuple 3");
                    },
                    |r| {
                        r.update_parse_on_delete(&rope_clone, char_idx_begin, char_idx_end);
                    },
                );

                true
            }
            Err(e) => {
                warn!("failed removing char {:?}-{:?} because {}", char_idx_begin, char_idx_end, e);
                false
            }
        };

        debug_assert!(self.check_invariant());

        result
    }

    fn undo(&mut self) -> bool {
        debug!("UNDO pos {} len {}", self.history_pos, self.history.len());
        let result = if self.history_pos > 0 {
            self.history_pos -= 1;
            true
        } else {
            false
        };

        debug_assert!(self.check_invariant());

        result
    }

    fn mark_as_saved(&mut self) {
        self.last_save_pos = Some(self.history_pos);
        debug_assert!(self.check_invariant());
    }

    fn is_saved(&self) -> bool {
        self.last_save_pos == Some(self.history_pos)
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

impl HasInvariant for BufferState {
    fn check_invariant(&self) -> bool {
        if self.history_pos >= self.history.len() {
            return false;
        }

        if let Some(last_save_pos) = self.last_save_pos {
            if last_save_pos >= self.history.len() {
                return false;
            }
        }

        true
    }
}
