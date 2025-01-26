use std::cmp::Ordering;
use std::collections::HashMap;
use std::slice::{Iter, IterMut};

use log::{error, warn};

use crate::cursor::cursor::{
    default_word_determinant, BackwardWordDeterminant, Cursor, CursorStatus, ForwardWordDeterminant, NEWLINE_WIDTH, ZERO_CURSOR,
};
use crate::primitives::has_invariant::HasInvariant;
use crate::text::text_buffer::TextBuffer;

// INVARIANTS:
// - non-empty this is so I can use cursor for anchoring and call "supercursor" easily
// - cursors are distinct
// - cursors have their anchors either on begin or on end, and they all have the anchor on the same
//   side
// - cursors DO NOT OVERLAP
// - cursors are SORTED by their anchor

// TODO add invariants:
// - sort cursors by anchor (they don't overlap, so it's easy)
// - (maybe) add "supercursor", which is always the first or the last, depending on which direction they were moved.
//      it would help with anchoring.

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct CursorSet {
    set: Vec<Cursor>,
}

impl CursorSet {
    pub fn single() -> Self {
        CursorSet {
            set: vec![Cursor::single()],
        }
    }

    pub fn new(set: Vec<Cursor>) -> Self {
        CursorSet { set }
    }

    /*
    This is singleton in set theory sense, not "design pattern"
     */
    pub fn singleton(cursor: Cursor) -> Self {
        CursorSet { set: vec![cursor] }
    }

    pub fn set(&self) -> &Vec<Cursor> {
        &self.set
    }

    pub fn set_mut(&mut self) -> &mut Vec<Cursor> {
        &mut self.set
    }

    // Returns only element OR None if the set is NOT a singleton.
    pub fn as_single(&self) -> Option<Cursor> {
        if self.set.len() != 1 {
            None
        } else {
            self.set.first().copied()
        }
    }

    pub fn as_single_mut(&mut self) -> Option<&mut Cursor> {
        if self.set.len() != 1 {
            None
        } else {
            self.set.first_mut()
        }
    }

    // Returns largest index either under the cursor or *within* selection.
    pub fn max_cursor_pos(&self) -> usize {
        let mut max: usize = 0;
        for c in self.set.iter() {
            max = usize::max(max, c.a);

            if let Some(sel) = c.s {
                debug_assert!(sel.b < sel.e);
                max = usize::max(max, usize::max(sel.b, sel.e));
            }
        }

        max
    }

    pub fn move_left(&mut self, selecting: bool) -> bool {
        self.move_left_by(1, selecting)
    }

    pub fn move_left_by(&mut self, l: usize, selecting: bool) -> bool {
        let mut res = false;

        for c in self.set.iter_mut() {
            if c.a > 0 {
                let old_pos = c.a;
                c.a -= std::cmp::min(c.a, l);

                if selecting {
                    c.update_select(old_pos, c.a);
                    c.preferred_column = None;
                } else {
                    c.clear_both();
                };

                res = true;
            }
        }

        // TODO test
        if selecting {
            // we will test for overlaps, and cut them. Since this is a move left, we cut a piece
            // from right side.

            for i in 0..self.set.len() - 1 {
                let right = self.set[i + 1];
                let left = &mut self.set[i];

                if let (Some(left_s), Some(right_s)) = (&mut left.s, right.s) {
                    // always remember: left index inclusive, right exclusive.
                    if left_s.e > right_s.b {
                        left_s.e = right_s.b;

                        if left_s.b >= left_s.e {
                            left.s = None;
                        }
                    }
                }
            }
        }

        self.reduce_left();

        res
    }

    pub fn move_right(&mut self, rope: &dyn TextBuffer, selecting: bool) -> bool {
        self.move_right_by(rope, 1, selecting)
    }

    pub fn move_right_by(&mut self, rope: &dyn TextBuffer, l: usize, selecting: bool) -> bool {
        if self.max_cursor_pos() > rope.len_chars() {
            error!("buffer shorter than cursor positions. Returning prematurely to avoid crash.");
            return false;
        }

        let len = rope.len_chars();
        let mut res = false;

        for c in &mut self.set {
            //we allow anchor after last char (so you can backspace last char)
            if c.a < len {
                let old_pos = c.a;
                c.a = std::cmp::min(c.a + l, len);

                if selecting {
                    c.update_select(old_pos, c.a);
                } else {
                    c.clear_both();
                }

                res = true;
            };
        }

        if selecting {
            // we will test for overlaps, and cut them. Since this is a move right, we cut a piece
            // from left side. I proceed in reverse order, so if setting selection to None I don't
            // destroy data I need to use in next pair, introducing glittering.
            for i in (0..self.set.len() - 1).rev() {
                let left = self.set[i];
                let right = &mut self.set[i + 1];

                if let (Some(left_s), Some(right_s)) = (left.s, &mut right.s) {
                    // always remember: left index inclusive, right exclusive.
                    if left_s.e > right_s.b {
                        right_s.b = left_s.e;

                        if right_s.b >= right_s.e {
                            right.s = None;
                        }
                    }
                }
            }
        }

        self.reduce_right();

        res
    }

    pub fn move_vertically_by(&mut self, rope: &dyn TextBuffer, l: isize, selecting: bool) -> bool {
        if self.max_cursor_pos() > rope.len_chars() {
            error!("buffer shorter than cursor positions. Returning prematurely to avoid crash.");
            return false;
        }

        if l == 0 {
            return false;
        }

        let mut res = false;

        let last_line_idx = rope.len_lines() - 1;

        for c in &mut self.set {
            //getting data
            if !selecting {
                c.clear_selection();
            }

            let cur_line_idx = match rope.char_to_line(c.a) {
                Some(line) => line,
                None => {
                    // If c.a == rope.len_chars(), rope (which is underlying impl) does not fail
                    // (see rope_tests.rs). Otherwise we would have to check whether last character
                    // is in fact a newline or not, as this would affect the result.
                    rope.len_lines()
                }
            };

            let cur_line_begin_char_idx = match rope.line_to_char(cur_line_idx) {
                Some(idx) => idx,
                None => {
                    error!("rope.line_to_char failed unexpectedly (1), skipping cursor.");
                    continue;
                }
            };
            let current_col_idx = c.a - cur_line_begin_char_idx;

            // line beyond the end of buffer
            if cur_line_idx as isize + l > last_line_idx as isize {
                if c.a == rope.len_chars() {
                    continue;
                }

                c.preferred_column = Some(current_col_idx);
                let old_pos = c.a;
                c.a = rope.len_chars(); // pointing to index higher than last valid one.

                if selecting {
                    c.update_select(old_pos, c.a);
                }

                res = true;
                continue;
            }

            // can't scroll that much up, begin of file is best we can get.
            if cur_line_idx as isize + l < 0 {
                if c.a == 0 {
                    continue;
                }

                c.preferred_column = Some(current_col_idx);
                let old_pos = c.a;
                c.a = 0;

                if selecting {
                    c.update_select(old_pos, c.a);
                }

                res = true;
                continue;
            }

            // at this point we know that 0 <= cur_line_idx <= last_line_idx
            debug_assert!(cur_line_idx <= last_line_idx);
            let new_line_idx = (cur_line_idx as isize + l) as usize;

            // This is actually right. Ropey counts '\n' as last character of current line.
            let last_char_in_new_line_idx = if new_line_idx == last_line_idx {
                //this corresponds to a notion of "potential new character" beyond the buffer. It's a valid cursor
                // position.
                rope.len_chars()
            } else {
                match rope.line_to_char(new_line_idx + 1) {
                    Some(char_idx) => char_idx - NEWLINE_WIDTH as usize,
                    None => {
                        error!("rope.line_to_char failed unexpectedly (2), skipping cursor.");
                        continue;
                    }
                }
            };

            let new_line_begin = match rope.line_to_char(new_line_idx) {
                Some(char_idx) => char_idx,
                None => {
                    error!("rope.line_to_char failed unexpectedly (3), skipping cursor.");
                    continue;
                }
            };

            let _new_line_is_last = new_line_idx + 1 == rope.len_lines();
            let new_line_num_chars = last_char_in_new_line_idx - new_line_begin;

            let preferred_target_column = match c.preferred_column {
                Some(pc) => pc,
                None => current_col_idx,
            };

            let old_pos = c.a;

            if new_line_num_chars >= preferred_target_column {
                c.clear_pc();
                c.a = new_line_begin + preferred_target_column;
            } else {
                c.a = new_line_begin + new_line_num_chars;
                if c.preferred_column.is_none() {
                    c.preferred_column = Some(current_col_idx);
                    debug_assert!(current_col_idx > 0);
                }
            }

            if selecting {
                c.update_select(old_pos, c.a);
            }

            if old_pos != c.a {
                res = true;
            }

            debug_assert!(
                c.a <= rope.len_chars(),
                "somehow put the cursor {:?} too far out (len_chars = {})",
                c,
                rope.len_chars()
            );
        }

        if l < 0 {
            self.reduce_left();
        } else {
            self.reduce_right();
        }

        res
    }

    pub fn get_cursor_status_for_char(&self, char_idx: usize) -> CursorStatus {
        let mut current_status = CursorStatus::None;

        for i in self.set.iter() {
            let new_status = i.get_cursor_status_for_char(char_idx);

            if new_status == CursorStatus::WithinSelection && current_status == CursorStatus::None {
                current_status = new_status;
            }

            if new_status == CursorStatus::UnderCursor {
                current_status = new_status;
                break;
            }
        }

        current_status
    }

    pub fn iter(&self) -> Iter<'_, Cursor> {
        self.set.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Cursor> {
        self.set.iter_mut()
    }

    // Returns FALSE if results in no-op
    pub fn move_home(&mut self, rope: &dyn TextBuffer, selecting: bool) -> bool {
        if self.max_cursor_pos() > rope.len_chars() {
            error!("buffer shorter than cursor positions. Returning prematurely to avoid crash.");
            return false;
        }

        let mut res = false;

        for c in self.set.iter_mut() {
            res |= c.move_home(rope, selecting);
        }

        self.reduce_left();

        res
    }

    // Returns FALSE if results in noop.
    pub fn move_end(&mut self, rope: &dyn TextBuffer, selecting: bool) -> bool {
        if self.max_cursor_pos() > rope.len_chars() {
            error!("buffer shorter than cursor positions. Returning prematurely to avoid crash.");
            return false;
        }

        let mut res = false;

        for c in self.set.iter_mut() {
            res |= c.move_end(rope, selecting);
        }

        // reducing - we just pick ones that are furthest left
        self.reduce_right();

        res
    }

    // this is a helper function, that moves cursors' .a to begin or end of selection.
    // returns whether operation had any impact on set or not.
    fn normalize_anchor(&mut self, right: bool) -> bool {
        let mut changed = false;
        for cursor in self.set.iter_mut() {
            if let Some(s) = cursor.s {
                debug_assert!(cursor.a == s.b || cursor.a == s.e);

                if right {
                    if cursor.a != s.e {
                        changed = true;
                    }
                    cursor.a = s.e;
                } else {
                    if cursor.a != s.b {
                        changed = true;
                    }
                    cursor.a = s.b;
                }
            }
        }
        changed
    }

    // Reduces cursors after a move left.
    // Moves anchors left.
    // When two anchors collide, keeps the one with longer selection.
    // When anchors are different, but selections overlap, I SHORTEN THE EARLIER SELECTION, because
    // I assume there have been a move LEFT with selection on.
    // Returns true if set was modified
    pub fn reduce_left(&mut self) -> bool {
        if self.set.len() == 1 {
            return false;
        }
        let mut res = false;
        res = self.normalize_anchor(false);
        if res {
            warn!("normalizing anchor left had an effect, this is not expected.");
        }

        let mut new_set = HashMap::<usize, Cursor>::new();

        self.set.sort_by_key(|c| c.a);

        for c in self.set.iter() {
            match new_set.get(&c.a) {
                None => {
                    new_set.insert(c.a, *c);
                }
                Some(old_c) => {
                    // we replace only if old one has shorter selection than new one.
                    match (old_c.s, c.s) {
                        (Some(old_sel), Some(new_sel)) => {
                            if old_sel.e < new_sel.e {
                                new_set.insert(c.a, *c);
                                res = true;
                            }
                        }
                        // if previous one had no selection, we consider new selection longer.
                        (None, Some(_new_sel)) => {
                            new_set.insert(c.a, *c);
                            res = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        if new_set.len() < self.set.len() {
            self.set.clear();
            for (_a, c) in new_set.iter() {
                self.set.push(*c);
            }
            self.set.sort_by_key(|c| c.a);
        }

        // now possibly shortening the selections.
        if self.set.len() > 1 {
            for i in 0..self.set.len() - 1 {
                let next = self.set[i + 1];
                let curr = &mut self.set[i];

                match &mut curr.s {
                    Some(curr_s) => {
                        // it's a little easier because I know from above sorts, that curr.a < next.a
                        if curr_s.e > next.a {
                            curr_s.e = next.a;
                            res = true;
                        }
                    }
                    None => {}
                }
            }
        }

        debug_assert!(!self.set.is_empty());
        res
    }

    // Reduces cursors after a move right.
    // Moves anchors right.
    // When two anchors collide, keeps the one with longer selection.
    // When anchors are different, but selections overlap, I SHORTEN THE LATER SELECTION, because
    // I assume there have been a move RIGHT with selection on.
    // Return true if set was modified
    pub fn reduce_right(&mut self) -> bool {
        let mut res = false;
        if self.set.len() == 1 {
            return res;
        }

        let norm_res = self.normalize_anchor(true);
        if norm_res {
            warn!("normalizing anchor right had an effect, this is not expected.");
        }

        let mut new_set = HashMap::<usize, Cursor>::new();

        self.set.sort_by_key(|c| c.a);

        for c in self.set.iter().rev() {
            match new_set.get(&c.a) {
                None => {
                    new_set.insert(c.a, *c);
                }
                Some(old_c) => {
                    // we replace only if old one has shorter selection than new one.
                    match (old_c.s, c.s) {
                        (Some(old_sel), Some(new_sel)) => {
                            if old_sel.b > new_sel.b {
                                new_set.insert(c.a, *c);
                                res = true;
                            }
                        }
                        // if previous one had no selection, we consider new selection longer.
                        (None, Some(_new_sel)) => {
                            new_set.insert(c.a, *c);
                            res = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        if new_set.len() < self.set.len() {
            self.set.clear();
            for (_a, c) in new_set.iter() {
                self.set.push(*c);
            }
            self.set.sort_by_key(|c| c.a);
        }

        // now possibly shortening the selections.
        for i in (1..self.set.len()).rev() {
            let prev = self.set[i - 1];
            let curr = &mut self.set[i];

            if let (Some(curr_s), Some(prev_s)) = (&mut curr.s, prev.s) {
                // it's a little easier because I know from above sorts, that curr.a < next.a
                if curr_s.b < prev_s.e {
                    curr_s.b = prev_s.e;
                    debug_assert!(curr.a == curr_s.e);
                    res = true;
                }
            }
        }

        debug_assert!(!self.set.is_empty());
        res
    }

    pub fn word_begin(&mut self, buffer: &dyn TextBuffer, selecting: bool, word_determinant: &BackwardWordDeterminant) -> bool {
        let mut res = false;

        for c in self.set.iter_mut() {
            res |= c.word_begin(buffer, selecting, word_determinant);
        }

        self.reduce_left();

        res
    }

    pub fn word_end(&mut self, buffer: &dyn TextBuffer, selecting: bool, word_determinant: &ForwardWordDeterminant) -> bool {
        let mut res = false;

        for c in self.set.iter_mut() {
            res |= c.word_end(buffer, selecting, word_determinant);
        }

        self.reduce_right();

        res
    }

    pub fn word_begin_default(&mut self, buffer: &dyn TextBuffer, selecting: bool) -> bool {
        self.word_begin(buffer, selecting, &default_word_determinant)
    }

    pub fn word_end_default(&mut self, buffer: &dyn TextBuffer, selecting: bool) -> bool {
        self.word_end(buffer, selecting, &default_word_determinant)
    }

    /*
    Drops selection and preferred column, used while jumping to "dropping cursor mode"
     */
    pub fn simplify(&mut self) -> bool {
        let mut res = false;
        for c in &mut self.set {
            res |= c.simplify();
        }
        res
    }

    /*
    Considers only selection, ignores preferred column.
     */
    pub fn are_simple(&self) -> bool {
        for c in &self.set {
            if !c.is_simple() {
                return false;
            }
        }
        true
    }

    /*
    Adds cursor, returns true if such cursor did not exist.
     */
    pub fn add_cursor(&mut self, cursor: Cursor) -> bool {
        debug_assert!(self.are_simple());
        if self.get_cursor_status_for_char(cursor.a) == CursorStatus::None {
            self.set.push(cursor);
            self.set.sort();

            debug_assert!(self.check_invariant());
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn remove_by_anchor(&mut self, anchor_char: usize) -> bool {
        for i in 0..self.set.len() {
            if self.set[i].a == anchor_char {
                self.set.remove(i);
                // disabled, we now allow "temporarily" empty sets in processing shift_tab
                //debug_assert!(self.check_invariant());
                return true;
            }
        }

        false
    }

    pub fn supercursor(&self) -> &Cursor {
        //TODO this should vary, depending on which direction cursors were moved last. Now it just points
        // to first one.

        // this succeeds, because of invariants
        self.set.first().unwrap_or_else(|| {
            error!("invariant broken, empty cursor_set.");
            &ZERO_CURSOR
        })
    }

    pub fn is_single(&self) -> bool {
        self.set.len() == 1
    }

    pub fn first(&self) -> Cursor {
        // TODO unwrap
        *self.set.first().unwrap()
    }
}

impl HasInvariant for CursorSet {
    fn check_invariant(&self) -> bool {
        // at least one
        if self.set.is_empty() {
            error!("cursor_set empty");
            return false;
        }

        for c in &self.set {
            if !c.check_invariant() {
                return false;
            }
        }

        // sorted, and anchors on the same side
        // TODO change to is_sorted once stabilized
        for idx in 1..self.set.len() {
            if self.set[idx - 1].cmp(&self.set[idx]) != Ordering::Less {
                error!(
                    "cursor[{}] = {:?} >= {:?} = cursor[{}]",
                    idx - 1,
                    self.set[idx - 1],
                    self.set[idx],
                    idx
                );
                return false;
            }
        }

        let mut anchor_left = false;
        let mut anchor_right = false;
        for c in &self.set {
            anchor_left |= c.anchor_left();
            anchor_right |= c.anchor_right();
        }
        if anchor_left && anchor_right {
            error!("invariant \"anchors on the same side\" failed.");
            return false;
        }

        // at this point I know they are sorted and anchor-aligned. All I need to do is to check if begin is
        // after previous end.
        for idx in 1..self.set.len() {
            if self.set[idx - 1].get_end() > self.set[idx].get_begin() {
                error!(
                    "cursor[{}].get_end() = {} > {} = cursor[{}].get_begin()",
                    idx - 1,
                    self.set[idx - 1].get_end(),
                    self.set[idx].get_begin(),
                    idx
                );
                return false;
            }
        }

        true
    }
}
