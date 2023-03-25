use std::sync::Arc;

use log::{debug, error};
use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;

use crate::experiments::regex_search::{FindError, regex_find};
use crate::primitives::cursor::Selection;
use crate::primitives::cursor::Cursor;
use crate::primitives::cursor_set::CursorSet;
use crate::primitives::search_pattern::SearchPattern;
use crate::tsw::lang_id::LangId;
use crate::tsw::parsing_tuple::ParsingTuple;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;

#[derive(Clone, Debug, Default)]
pub struct ContentsAndCursors {
    pub rope: Rope,
    pub parsing: Option<ParsingTuple>,
    pub cursor_set: CursorSet,
}

impl ContentsAndCursors {
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

    pub fn parse(&mut self, tree_sitter: Arc<TreeSitterWrapper>, lang_id: LangId) -> bool {
        if let Some(parsing_tuple) = tree_sitter.new_parse(lang_id) {
            self.parsing = Some(parsing_tuple);

            true
        } else {
            false
        }
    }

    /*
    This is an action destructive to cursor set - it uses only the supercursor.anchor as starting point for
    search.

    returns Ok(true) iff there was an occurrence
     */
    pub fn find_once(&mut self, pattern: &str) -> Result<bool, FindError> {
        let start_pos = self.cursor_set.supercursor().a;
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

            self.cursor_set = new_cursors;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /*
    Returns true iff:
        - all cursors have selections
        - all selections match the pattern
     */
    pub fn do_cursors_match_regex(&self, pattern: &SearchPattern) -> bool {
        for c in self.cursor_set.iter() {
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
}

impl ToString for ContentsAndCursors {
    fn to_string(&self) -> String {
        self.rope.to_string()
    }
}