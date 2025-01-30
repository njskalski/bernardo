use flexi_logger::AdaptiveFormat::Default;
use log::{debug, error, warn};
use ropey::Rope;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;
use std::sync::RwLock;
use tree_sitter::{Node, QueryCursor};
use unicode_segmentation::UnicodeSegmentation;

use crate::cursor::cursor::Cursor;
use crate::cursor::cursor::Selection;
use crate::cursor::cursor_set::CursorSet;
use crate::experiments::regex_search::{regex_find, FindError};
use crate::primitives::has_invariant::HasInvariant;
use crate::primitives::search_pattern::SearchPattern;
use crate::text::ident_type::IndentType;
use crate::tsw::lang_id::LangId;
use crate::tsw::parsing_tuple::ParsingTuple;
use crate::tsw::rope_wrappers::RopeWrapper;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::widget::widget::WID;
use crate::widgets::editor_widget::label::label::Label;
use crate::{unpack, unpack_or, unpack_or_e};
/*
I allow empty history, it means "nobody is looking at the buffer now, first who comes needs to set
it's cursors.
 */

#[derive(Clone, Debug)]
pub struct ContentsAndCursors {
    rope: Rope,
    parsing: Option<ParsingTuple>,
    cursor_sets: Vec<(WID, CursorSet)>,
    labels: Vec<Label>,
}

impl ContentsAndCursors {
    pub fn new(rope: Rope, parsing: Option<ParsingTuple>) -> Self {
        ContentsAndCursors {
            rope,
            parsing,
            cursor_sets: Vec::new(),
            labels: Vec::new(),
        }
    }

    pub fn add_cursor_set(&mut self, widget_id: WID, cs: CursorSet) -> bool {
        if self.cursor_sets.iter().find(|(wid, _)| *wid == widget_id).is_some() {
            error!(
                "can't add cursor set for WidgetID {} - it's already present. Did you mean 'set_cursor_set'?",
                widget_id
            );
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
        let cursor_set = unpack_or_e!(
            self.get_cursor_set(widget_id),
            false,
            "can't find cursor set for WidgetID {}",
            widget_id
        );

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

    // TODO remove "empty"
    pub fn empty() -> Self {
        ContentsAndCursors {
            rope: Rope::default(),
            parsing: None,
            cursor_sets: vec![],
            labels: vec![],
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
        let cursor_set = unpack_or_e!(
            self.get_cursor_set(widget_id),
            Err(FindError::WidgetIdNotFound),
            "WidgetId not found"
        );

        let start_pos = cursor_set.supercursor().a;
        let mut matches = regex_find(pattern, &self.rope, Some(start_pos))?;

        if let Some(m) = matches.next() {
            if m.0 == m.1 {
                error!("empty find, this should not be possible");
                return Ok(false);
            }

            let new_cursors = CursorSet::singleton(Cursor::new(m.1).with_selection(Selection::new(m.0, m.1)));

            self.set_cursor_set(widget_id, new_cursors);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_cursor_set(&self, widget_id: WID) -> Option<&CursorSet> {
        let res = self.cursor_sets.iter().find(|(wid, _)| *wid == widget_id).map(|(_, cs)| cs);

        if res.is_none() {
            let cursors = self.cursor_sets.iter().map(|item| item.0).collect::<Vec<_>>();
            error!("failed to find WID {:?} in Cursors, available are: {:?}", widget_id, cursors)
        }

        res
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

    pub fn parsing(&self) -> Option<&ParsingTuple> {
        self.parsing.as_ref()
    }

    pub fn parsing_mut(&mut self) -> Option<&mut ParsingTuple> {
        self.parsing.as_mut()
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn rope_mut(&mut self) -> &mut Rope {
        &mut self.rope
    }

    pub fn has_cursor_set_for(&self, widget_id: WID) -> bool {
        self.cursor_sets.iter().find(|(wid, _)| *wid == widget_id).is_some()
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
        Self { rope, ..self }
    }

    pub fn labels(&self) -> &[Label] {
        &self.labels
    }

    pub fn replace_labels(&mut self, labels: Vec<Label>) {
        self.labels = labels;
    }

    // TODO this is dead code, I don't know how to fix it, I don't understand treesitter enough
    pub fn get_indentation_level_with_treesitter(&self, cursor: &Cursor) -> Option<usize> {
        let parsing_tuple = self.parsing.as_ref()?;
        let query = parsing_tuple.indent_query.as_ref()?;
        let cursor_in_bytes = unpack_or_e!(self.rope.try_char_to_byte(cursor.a).ok(), None, "failed to convert cursor to bytes");
        let tree = parsing_tuple.tree.as_ref()?;

        let node = tree.root_node().descendant_for_byte_range(cursor_in_bytes, cursor_in_bytes + 1)?;

        let mut cursor = QueryCursor::new();

        let mut nodes_on_path: HashSet<Node> = HashSet::new();

        let mut node_iter = tree.root_node().descendant_for_byte_range(cursor_in_bytes, cursor_in_bytes);

        while let Some(node) = node_iter {
            nodes_on_path.insert(node);
            node_iter = node.parent();
        }

        // let query_matches = cursor.matches(&query, node, RopeWrapper(&self.rope));
        let query_captures = cursor.captures(&query, node, RopeWrapper(&self.rope));

        let mut name_to_num: HashMap<String, usize> = HashMap::new();
        let mut bs: Vec<_> = vec![];
        for (query_match, _) in query_captures {
            let mut result: Vec<_> = vec![];

            for capture in query_match.captures {
                // if !nodes_on_path.contains(&capture.node) {
                //     error!("skipping node");
                //     continue;
                // }

                let capture_name = query.capture_names()[capture.index as usize];

                if let Some(value) = name_to_num.get_mut(capture_name) {
                    *value += 1;
                } else {
                    name_to_num.insert(capture_name.to_string(), 1);
                }

                result.push(capture_name);
            }

            bs.push(result);
        }

        error!("query_matches: [{:?}], name_to_num = [{:?}]", bs, name_to_num);

        None
    }

    // This method retrieves indentation of current line under the cursor, returns (num_spaces, num_tabs)
    pub fn get_indentation_level_dumb(&self, cursor: &Cursor) -> (usize, usize) {
        let line_idx = unpack_or_e!(
            self.rope.try_char_to_line(cursor.a).ok(),
            (0, 0),
            "failed converting char_idx to line_idx"
        );
        debug_assert!(line_idx < self.rope.len_lines());
        let line = self.rope.line(line_idx);

        let mut num_spaces: usize = 0;
        let mut num_tabs: usize = 0;

        for char in line.chars() {
            match char {
                '\t' => num_tabs += 1,
                ' ' => num_spaces += 1,
                _ => break,
            }
        }

        (num_spaces, num_tabs)
    }

    // TODO can replace cs with WID
    pub fn get_common_indentation_level_for_cursor_set(&self, cs: &CursorSet) -> Option<(usize, IndentType)> {
        // TODO fix indent from TreeSitter where available

        let res = self.get_indentation_level_dumb(&cs.first());
        for c in cs.iter().skip(1) {
            let new_res = self.get_indentation_level_dumb(c);

            if res != new_res {
                return None;
            }
        }

        if res.0 > 0 && res.1 > 0 {
            debug!("found both spaces and tabs, don't know what to do, returning nothing");
            return None;
        }

        if res.0 > 0 {
            Some((res.0, IndentType::Spaces))
        } else if res.1 > 0 {
            Some((res.1, IndentType::Tabs))
        } else {
            Some((0, IndentType::Spaces))
        }
    }
}

impl ToString for ContentsAndCursors {
    fn to_string(&self) -> String {
        self.rope.to_string()
    }
}

impl HasInvariant for ContentsAndCursors {
    fn check_invariant(&self) -> bool {
        let len = self.rope.len_chars();

        for cs in &self.cursor_sets {
            if cs.1.check_invariant() == false {
                error!("cs.invariant");
                return false;
            }

            for c in cs.1.iter() {
                if c.check_invariant() == false {
                    error!("c.invariant");
                    return false;
                }

                if c.a > len {
                    error!("c.a > len {} {}", c.a, len);
                    return false;
                }
            }
        }

        true
    }
}
