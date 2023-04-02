use std::sync::Arc;

use log::{debug, error, warn};
use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;

use crate::cursor::cursor::Cursor;
use crate::cursor::cursor::Selection;
use crate::cursor::cursor_set::CursorSet;
use crate::experiments::regex_search::{FindError, regex_find};
use crate::io::buffer;
use crate::primitives::search_pattern::SearchPattern;
use crate::tsw::lang_id::LangId;
use crate::tsw::parsing_tuple::ParsingTuple;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::unpack_or_e;
use crate::widget::widget::WID;

/*
I allow empty history, it means "nobody is looking at the buffer now, first who comes needs to set
it's cursors.
 */

#[derive(Clone, Debug)]
pub struct ContentsAndCursors {
    pub rope: Rope,
    pub parsing: Option<ParsingTuple>,
    pub cursor_sets: Vec<(WID, CursorSet)>,
}

impl ContentsAndCursors {
    pub fn add_cursor_set(&mut self, widget_id: WID, cs: CursorSet) -> bool {
        if self.cursor_sets.iter().find(|(wid, _)| *wid == widget_id).is_some() {
            error!("can't add cursor set for WidgetID {} - it's already present. Did you mean 'set_cursor_set'?", widget_id);
            return false;
        }

        self.cursor_sets.push((widget_id, cs));
        true
    }

    /*
    Returns true iff:
        - all cursors have selections
        - all selections match the pattern
     */
    pub fn do_cursors_match_regex(&self, widget_id: WID, pattern: &SearchPattern) -> bool {
        let cursor_set = unpack_or_e!(self.get_cursor_set(widget_id), false, "can't find cursor set for WidgetID {}", widget_id);

        for c in cursor_set.iter() {
            if c.s.is_none() {
                return false;
            }
            let sel = c.s.unwrap();
            let selected: String = self.rope.chars().skip(sel.b).take(sel.e - sel.b).collect();

            if !pattern.matches(&selected) {
                return false;
            }
        }

        true
    }

    pub fn empty() -> Self {
        ContentsAndCursors {
            rope: Rope::default(),
            parsing: None,
            cursor_sets: vec![],
        }
    }

    pub fn ends_with_at(&self, char_offset: usize, what: &str) -> bool {
        let what_char_len = what.graphemes(true).count();

        if self.rope.len_chars() < char_offset {
            debug!("ends_wit_at beyond end");
            return false;
        }
        let sub_rope = self.rope.slice(0..char_offset);
        let rope_len = sub_rope.len_chars();

        if rope_len < what_char_len {
            false
        } else {
            let mut tail = String::new();
            for char_idx in 0..what_char_len {
                match sub_rope.get_char(rope_len - what_char_len + char_idx) {
                    Some(ch) => {
                        tail.push(ch);
                    }
                    None => {
                        error!("failed unwrapping expected character");
                        return false;
                    }
                }
            }

            debug_assert!(tail.graphemes(true).count() == what_char_len);

            what == &tail
        }
    }

    /*
    This is an action destructive to cursor set - it uses only the supercursor.anchor as starting
    point for search.

    returns Ok(true) iff there was an occurrence
     */
    pub fn find_once(&mut self, widget_id: WID, pattern: &str) -> Result<bool, FindError> {
        let cursor_set = unpack_or_e!(self.get_cursor_set(widget_id), Err(FindError::WidgetIdNotFound), "WidgetId not found");

        let start_pos = cursor_set.supercursor().a;
        let mut matches = regex_find(
            pattern,
            &self.rope,
            Some(start_pos),
        )?;

        if let Some(m) = matches.next() {
            if m.0 == m.1 {
                error!("empty find, this should not be possible");
                return Ok(false);
            }

            let new_cursors = CursorSet::singleton(
                Cursor::new(m.1).with_selection(Selection::new(m.0, m.1))
            );

            self.set_cursor_set(widget_id, new_cursors);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_cursor_set(&self, widget_id: WID) -> Option<&CursorSet> {
        self.cursor_sets.iter().find(|(wid, _)| *wid == widget_id).map(|(_, cs)| cs)
    }

    pub fn get_cursor_set_mut(&mut self, widget_id: WID) -> Option<&mut CursorSet> {
        self.cursor_sets.iter_mut().find(|(wid, _)| *wid == widget_id).map(|(_, cs)| cs)
    }

    pub fn parse(&mut self, tree_sitter: Arc<TreeSitterWrapper>, lang_id: LangId) -> bool {
        if let Some(parsing_tuple) = tree_sitter.new_parse(lang_id) {
            self.parsing = Some(parsing_tuple);

            true
        } else {
            false
        }
    }

    pub fn set_cursor_set(&mut self, widget_id: WID, cursor_set: CursorSet) -> bool {
        match self.get_cursor_set_mut(widget_id) {
            Some(old_cs) => {
                *old_cs = cursor_set;
                true
            }
            None => {
                error!("NOT setting cursor_set for previously unknown WidgetID {:?}, this is most likely an error. Did you mean 'add_cursor_set'?", widget_id);
                false
            }
        }
    }

    pub fn with_cursor_set(mut self, wid_and_cursor_set: (WID, CursorSet)) -> Self {
        self.cursor_sets.push(wid_and_cursor_set);
        self
    }

    pub fn with_rope(self, rope: Rope) -> Self {
        Self {
            rope,
            ..self
        }
    }
}

impl ToString for ContentsAndCursors {
    fn to_string(&self) -> String {
        self.rope.to_string()
    }
}