// Copyright 2018-2020 Google LLC, 2021-2022 Andrzej J Skalski
// This version of the file (2021+) is licensed with GNU LGPLv3 License.
// For older version of file (licensed under Apache 2 license), see sly-editor, at
// https://github.com/njskalski/sly-editor/blob/master/src/cursor_set.rs

// Cursor == (Selection, Anchor), thanks Kakoune!
// both positions and anchor are counted in CHARS not offsets.
// Furthermore, I impose a following invariant: the anchor is always above one of selection ends.

// The cursor points to a index where a NEW character will be included, or old character will be
// REPLACED.

// Cursor pointing to a newline character is visualized as an option to append preceding it line.

// So Cursor can point 1 character BEYOND length of buffer!

// Newline is always an end of previous line, not a beginning of new.

// TODO change the selection to Option<usize> to ENFORCE the invariant by reducing the volume of data.
// TODO decide: introduce a special cursor to derive from while in "dropping_cursor" mode?
// TODO add "invariant protectors" to cursor set and warnings/errors, maybe add tests.

// INVARIANTS:
// - non-empty
//      this is so I can use cursor for anchoring and call "supercursor" easily
// - cursors are distinct
// - cursors have their anchors either on begin or on end, and they all have the anchor on the same side
// - cursors DO NOT OVERLAP
// - cursors are SORTED by their anchor

// TODO add invariants:
// - sort cursors by anchor (they don't overlap, so it's easy)
// - (maybe) add "supercursor", which is always the first or the last, depending on which direction they were moved.
//      it would help with anchoring.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Range;
use std::slice::{Iter, IterMut};

use log::{error, warn};

use crate::text::text_buffer::TextBuffer;

const NEWLINE_LENGTH: usize = 1; // TODO(njskalski): add support for multisymbol newlines?

const ZERO_CURSOR: Cursor = Cursor::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum CursorStatus {
    None,
    WithinSelection,
    UnderCursor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/*
    Describes a selection of text.
    Invariant: anchor is at begin OR end, never in between.
 */
pub struct Selection {
    //begin inclusive
    pub b: usize,
    //end EXCLUSIVE (as *everywhere*)
    pub e: usize,
}

impl Selection {
    pub fn new(b: usize, e: usize) -> Self {
        //TODO got a panic here with move_vertically_by on cursor up
        debug_assert!(b < e, "b {} e {}", b, e);
        Selection {
            b,
            e,
        }
    }

    pub fn within(&self, char_idx: usize) -> bool {
        char_idx >= self.b && char_idx < self.e
    }

    pub fn len(&self) -> usize {
        debug_assert!(self.b < self.e);
        if self.b >= self.e {
            error!("selection with begin > end, returning 0 for length: {:?}", self);
            0
        } else {
            self.e - self.b
        }
    }
}

impl PartialOrd<Self> for Selection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for Selection {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.b, &other.b).then(
            Ord::cmp(&self.e, &other.e)
        )
    }
}

// both signatures are buffer, idx 
pub type ForwardWordDeterminant = Fn(&dyn TextBuffer, usize, usize) -> bool;
pub type ReverseWordDeterminant = Fn(&dyn TextBuffer, usize, usize) -> bool;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Cursor {
    // selection. Invariant: anchor is either at begin or end of selection, never inside.
    pub s: Option<Selection>,
    // anchor (position)
    pub a: usize,
    pub preferred_column: Option<usize>,
}

impl Cursor {
    pub fn single() -> Self {
        Cursor {
            s: None,
            a: 0,
            preferred_column: None,
        }
    }

    pub const fn new(anc: usize) -> Self {
        Cursor {
            s: None,
            a: anc,
            preferred_column: None,
        }
    }

    pub fn with_selection(self, selection: Selection) -> Self {
        debug_assert!(selection.b == self.a || selection.e == self.a);

        let a = if selection.e == self.a || selection.b == self.a {
            self.a
        } else {
            warn!("Attempted setting selection not respecting invariant. Moving anchor to re-establish it.");
            selection.e
        };

        Cursor {
            s: Some(selection),
            a,
            ..self
        }
    }

    pub fn with_preferred_column(self, preferred_column: usize) -> Self {
        Cursor {
            preferred_column: Some(preferred_column),
            ..self
        }
    }

    pub fn shift_by(&mut self, shift: isize) -> bool {
        if shift < 0 {
            let abs_shift = shift.unsigned_abs();

            if self.a < abs_shift || self.s.map(|sel| sel.b < abs_shift).unwrap_or(false) {
                error!("attempted to substract {} from {:?}, ignoring completely.", shift, self);
                return false;
            }
        }

        self.a = (shift + self.a as isize) as usize;
        if let Some(sel) = self.s {
            self.s = Some(Selection::new((shift + sel.b as isize) as usize, (shift + sel.e as isize) as usize));
        }

        true
    }

    pub fn advance_and_clear(&mut self, advance_by: isize) -> bool {
        if advance_by < 0 && self.a < advance_by.unsigned_abs() {
            error!("attempted to substract {} from {}, using 0 as failsafe.", advance_by, self.a);
            self.a = 0;
            self.clear_both();
            false
        } else {
            // TODO overflow?
            self.a = (advance_by + self.a as isize) as usize;
            self.clear_both();
            true
        }
    }

    // Updates selection, had it changed.
    // old_pos is one of selection ends.
    // new_pos is where THAT end is moved.
    // if there is no selection, I behave as if 0-length selection at old_pos was present, so
    //      it gets expanded towards new_pos.
    pub fn update_select(&mut self, old_pos: usize, new_pos: usize) {
        match self.s {
            None => {}
            Some(s) => {
                debug_assert!(s.b == old_pos || s.e == old_pos);
            }
        }

        if old_pos == new_pos {
            return;
        }

        match self.s {
            None => {
                self.s = Some(Selection::new(
                    usize::min(old_pos, new_pos),
                    usize::max(old_pos, new_pos),
                ))
            }
            Some(sel) => {
                /* and here'd be dragons:
                   so I need to cover a following scenario:
                   [   ]
                    [  ]
                      []
                       |
                       []
                       [ ]
                       [  ]
                   So shift initially engaged at middle position, then user moved left while holding
                   it, then decided to go right. In this scenario, selection shrinks.
                */

                debug_assert!(old_pos == sel.b || old_pos == sel.e);

                if sel.b == old_pos {
                    if new_pos != sel.e {
                        self.s = Some(Selection::new(new_pos, sel.e));
                    } else {
                        self.s = None;
                    }

                    return;
                }
                if sel.e == old_pos {
                    if sel.b != new_pos {
                        self.s = Some(Selection::new(sel.b, new_pos));
                    } else {
                        self.s = None;
                    }

                    return;
                }
                error!("invariant that selection begins or ends with anchor broken. Not crashing, but fix it.");
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.s = None;
    }

    pub fn clear_pc(&mut self) {
        self.preferred_column = None;
    }

    // Clears both selection and preferred column.
    pub fn clear_both(&mut self) {
        self.s = None;
        self.preferred_column = None;
    }

    pub fn get_cursor_status_for_char(&self, char_idx: usize) -> CursorStatus {
        if char_idx == self.a {
            return CursorStatus::UnderCursor;
        }

        if self.s.is_some() {
            if self.s.unwrap().within(char_idx) {
                return CursorStatus::WithinSelection;
            }
        }

        CursorStatus::None
    }

    // Returns FALSE if noop.
    pub fn move_home(&mut self, rope: &dyn TextBuffer, selecting: bool) -> bool {
        let old_pos = self.a;
        let line = rope.char_to_line(self.a).unwrap(); //TODO
        let new_pos = rope.line_to_char(line).unwrap(); //TODO

        debug_assert!(new_pos <= old_pos);

        let res = if new_pos == self.a {
            // in this variant we are just clearing the preferred column. Any selection is not
            // important.
            if self.preferred_column.is_some() {
                self.preferred_column = None;

                true
            } else {
                false
            }
        } else {
            self.a = new_pos;
            if selecting {
                self.update_select(new_pos, old_pos);
            } else {
                self.clear_selection();
            }

            self.preferred_column = None;

            true
        };

        res
    }

    // Returns FALSE if noop.
    pub fn move_end(&mut self, rope: &dyn TextBuffer, selecting: bool) -> bool {
        let old_pos = self.a;
        let next_line = rope.char_to_line(self.a).unwrap() + 1; // TODO

        let new_pos = if rope.len_lines() > next_line {
            rope.line_to_char(next_line).unwrap() - 1 //TODO
        } else {
            rope.len_chars() // yes, one beyond num chars
        };

        debug_assert!(new_pos >= old_pos);

        let res = if new_pos == self.a {
            // in this variant we are just clearing the preferred column. Any selection is not
            // important.
            if self.preferred_column.is_some() {
                self.preferred_column = None;

                true
            } else {
                false
            }
        } else {
            self.a = new_pos;
            if selecting {
                self.update_select(old_pos, new_pos);
            } else {
                self.clear_selection();
            }
            self.preferred_column = None;

            true
        };

        debug_assert!(self.check_invariant());

        res
    }

    // Returns FALSE on noop.
    // word_determinant should return FALSE when word ends, and TRUE while it continues.
    fn word_begin<F: Fn(usize) -> bool>(&mut self, selecting: bool, word_determinant: F) -> bool {
        if self.a == 0 {
            return false;
        }

        let old_pos = self.a;

        if self.a > 0 {
            if word_determinant(self.a - 1) {
                // case when cursor is within a word
                while self.a > 0 && word_determinant(self.a - 1) {
                    self.a -= 1;
                }
            } else {
                // otherwise we do just one step.
                self.a -= 1; //safe to do, we checked it's > 0.
            }
        }


        if selecting {
            self.update_select(old_pos, self.a);
        } else {
            self.clear_selection();
        }

        debug_assert!(old_pos >= self.a);

        old_pos != self.a
    }

    fn word_end<F: FnMut(char, char) -> bool>(&mut self, buffer: &dyn TextBuffer, selecting: bool, word_determinant: F) -> bool {
        if self.a == buffer.len_chars() {
            return false;
        }

        let old_pos = self.a;

        if self.a < buffer.len_chars() {
            if word_determinant(buffer.char_at(self.a).unwrap()) {
                // variant within the word
                while self.a < buffer.len_chars() {
                    if word_determinant(buffer.char_at(self.a).unwrap()) {
                        self.a += 1;
                    } else {
                        break;
                    }
                }
            } else {
                self.a += 1;
            }
        }

        if selecting {
            self.update_select(old_pos, self.a);
        } else {
            self.clear_selection();
        }

        debug_assert!(old_pos <= self.a);

        old_pos != self.a
    }

    /*
    Drops selection and preferred column.
     */
    pub fn simplify(&mut self) -> bool {
        let mut res = false;
        if self.preferred_column.is_some() {
            self.preferred_column = None;
            res = true;
        }

        if self.s.is_some() {
            self.s = None;
            res = true;
        }

        res
    }

    /*
    This one IGNORES preferred column
     */
    pub fn is_simple(&self) -> bool {
        self.s.is_none()
    }

    pub fn check_invariant(&self) -> bool {
        if let Some(s) = self.s {
            s.b != s.e && (s.b == self.a || s.e == self.a)
        } else {
            true
        }
    }

    pub fn anchor_left(&self) -> bool {
        self.s.map(|s| s.b == self.a).unwrap_or(false)
    }

    pub fn anchor_right(&self) -> bool {
        self.s.map(|s| s.e == self.a).unwrap_or(false)
    }

    pub fn get_begin(&self) -> usize {
        self.s.map(|s| s.b).unwrap_or(self.a)
    }

    pub fn get_end(&self) -> usize {
        self.s.map(|s| s.e).unwrap_or(self.a)
    }

    // TODO tests
    pub fn intersects(&self, char_range: &Range<usize>) -> bool {
        if self.is_simple() {
            return char_range.start <= self.a && self.a < char_range.end;
        }

        // I will use simple "bracket" evaluation: true opens bracket, false closes bracket
        //  (because in case of idx collision we want to first close and then open)
        let mut brackets: Vec<(usize, bool)> = Vec::new();

        brackets.push((char_range.start, true));
        brackets.push((char_range.end, false));
        brackets.push((self.get_begin(), true));
        brackets.push((self.get_end(), false));

        brackets.sort();

        let mut how_many_open_brackets: u8 = 0;
        for b in brackets {
            if b.1 {
                how_many_open_brackets += 1;
            } else {
                how_many_open_brackets -= 1;
            }
            if how_many_open_brackets > 1 {
                return true;
            }
        }
        return false;
    }
}

impl PartialOrd<Self> for Cursor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for Cursor {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.a, &other.a).then(
            Ord::cmp(&self.s, &other.s).then(
                Ord::cmp(&self.preferred_column, &other.preferred_column)
            )
        )
    }
}

impl Into<Cursor> for (usize, usize, usize) {
    fn into(self) -> Cursor {
        Cursor {
            s: Some(Selection {
                b: self.0,
                e: self.1,
            }),
            a: self.2,
            preferred_column: None,
        }
    }
}

impl Into<Cursor> for usize {
    fn into(self) -> Cursor {
        Cursor {
            s: None,
            a: self,
            preferred_column: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct CursorSet {
    set: Vec<Cursor>,
}

impl Default for CursorSet {
    fn default() -> Self {
        CursorSet::single()
    }
}

impl CursorSet {
    pub fn single() -> Self {
        CursorSet {
            set: vec![Cursor::single()],
        }
    }

    #[cfg(test)]
    pub fn new(set: Vec<Cursor>) -> Self {
        CursorSet { set }
    }

    /*
    This is singleton in set theory sense, not "design pattern"
     */
    pub fn singleton(cursor: Cursor) -> Self {
        CursorSet {
            set: vec![cursor]
        }
    }

    pub fn set(&self) -> &Vec<Cursor> {
        &self.set
    }

    pub fn set_mut(&mut self) -> &mut Vec<Cursor> {
        &mut self.set
    }

    // Returns only element OR None if the set is NOT a singleton.
    pub fn as_single(&self) -> Option<&Cursor> {
        if self.set.len() != 1 {
            None
        } else {
            self.set.first()
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

            c.s.map(|sel| {
                debug_assert!(sel.b < sel.e);
                max = usize::max(max, usize::max(sel.b, sel.e));
            });
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

                match (&mut left.s, right.s) {
                    (Some(left_s), Some(right_s)) => {
                        // always remember: left index inclusive, right exclusive.
                        if left_s.e > right_s.b {
                            left_s.e = right_s.b;

                            if left_s.b >= left_s.e {
                                left.s = None;
                            }
                        }
                    }
                    _ => {}
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

        for mut c in &mut self.set {
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

                match (left.s, &mut right.s) {
                    (Some(left_s), Some(right_s)) => {
                        // always remember: left index inclusive, right exclusive.
                        if left_s.e > right_s.b {
                            right_s.b = left_s.e;

                            if right_s.b >= right_s.e {
                                right.s = None;
                            }
                        }
                    }
                    _ => {}
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

        for mut c in &mut self.set {
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
                //this corresponds to a notion of "potential new character" beyond the buffer. It's a valid cursor position.
                rope.len_chars()
            } else {
                match rope.line_to_char(new_line_idx + 1) {
                    Some(char_idx) => char_idx - NEWLINE_LENGTH,
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

            let new_line_num_chars = last_char_in_new_line_idx + 1 - new_line_begin;

            if let Some(preferred_column) = c.preferred_column {
                // debug_assert!(preferred_column >= current_col_idx);

                let old_pos = c.a;

                if preferred_column <= (new_line_num_chars - NEWLINE_LENGTH) {
                    c.clear_pc();
                    c.a = new_line_begin + preferred_column;
                } else {
                    c.a = new_line_begin + new_line_num_chars - NEWLINE_LENGTH;
                }

                if selecting {
                    c.update_select(old_pos, c.a);
                }

                if old_pos != c.a {
                    res = true;
                }
            } else {
                let old_pos = c.a;

                let addon = if new_line_idx == last_line_idx { 1 } else { 0 };
                // inequality below is interesting.
                // The line with characters 012 is 3 characters long. So if current char idx is 3
                // it means that line below needs at least 4 character to host it without shift left.
                // "addon" is there to make sure that last line is counted as "one character longer"
                // than it actually is, so we can position cursor one character behind buffer
                // (appending).
                if new_line_num_chars + addon <= current_col_idx {
                    c.a = new_line_begin + new_line_num_chars - 1; //this -1 is needed.
                    c.preferred_column = Some(current_col_idx);
                } else {
                    c.a = new_line_begin + current_col_idx;
                }

                if selecting {
                    c.update_select(old_pos, c.a);
                }

                if old_pos != c.a {
                    res = true;
                }
            }
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
        };

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
        };

        // reducing - we just pick ones that are furthest left
        self.reduce_right();

        res
    }

    // this is a helper function, that moves cursors' .a to begin or end of selection.
    // returns whether operation had any impact on set or not.
    fn normalize_anchor(&mut self, right: bool) -> bool {
        let mut changed = false;
        for cursor in self.set.iter_mut() {
            match cursor.s {
                Some(s) => {
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
                None => {}
            }
        }
        changed
    }

    // Reduces cursors after a move left.
    // Moves anchors left.
    // When two anchors collide, keeps the one with longer selection.
    // When anchors are different, but selections overlap, I SHORTEN THE EARLIER SELECTION, because
    // I assume there have been a move LEFT with selection on.
    pub fn reduce_left(&mut self) {
        if self.set.len() == 1 {
            return;
        }

        let norm_res = self.normalize_anchor(false);
        if norm_res {
            warn!("normalizing anchor left had an effect, this is not expected.");
        }

        let mut new_set = HashMap::<usize, Cursor>::new();

        self.set.sort_by_key(|c| c.a);

        for c in self.set.iter() {
            match new_set.get(&c.a) {
                None => { new_set.insert(c.a, c.clone()); }
                Some(old_c) => {
                    // we replace only if old one has shorter selection than new one.
                    match (old_c.s, c.s) {
                        (Some(old_sel), Some(new_sel)) => {
                            if old_sel.e < new_sel.e {
                                new_set.insert(c.a, c.clone());
                            }
                        }
                        // if previous one had no selection, we consider new selection longer.
                        (None, Some(_new_sel)) => {
                            new_set.insert(c.a, c.clone());
                        }
                        _ => {}
                    }
                }
            }
        }

        if new_set.len() < self.set.len() {
            self.set.clear();
            for (_a, c) in new_set.iter() {
                self.set.push(c.clone());
            }
            self.set.sort_by_key(|c| c.a);
        }

        // now possibly shortening the selections.
        if self.set.len() > 1 {
            for i in 0..self.set.len() - 1 {
                let next = self.set[i + 1].clone();
                let curr = &mut self.set[i];

                match &mut curr.s {
                    Some(curr_s) => {
                        // it's a little easier because I know from above sorts, that curr.a < next.a
                        if curr_s.e > next.a {
                            curr_s.e = next.a;
                        }
                    }
                    None => {}
                }
            }
        }

        debug_assert!(!self.set.is_empty());
    }

    // Reduces cursors after a move right.
    // Moves anchors right.
    // When two anchors collide, keeps the one with longer selection.
    // When anchors are different, but selections overlap, I SHORTEN THE LATER SELECTION, because
    // I assume there have been a move RIGHT with selection on.
    pub fn reduce_right(&mut self) {
        if self.set.len() == 1 {
            return;
        }

        let norm_res = self.normalize_anchor(true);
        if norm_res {
            warn!("normalizing anchor right had an effect, this is not expected.");
        }

        let mut new_set = HashMap::<usize, Cursor>::new();

        self.set.sort_by_key(|c| c.a);

        for c in self.set.iter().rev() {
            match new_set.get(&c.a) {
                None => { new_set.insert(c.a, c.clone()); }
                Some(old_c) => {
                    // we replace only if old one has shorter selection than new one.
                    match (old_c.s, c.s) {
                        (Some(old_sel), Some(new_sel)) => {
                            if old_sel.b > new_sel.b {
                                new_set.insert(c.a, c.clone());
                            }
                        }
                        // if previous one had no selection, we consider new selection longer.
                        (None, Some(_new_sel)) => {
                            new_set.insert(c.a, c.clone());
                        }
                        _ => {}
                    }
                }
            }
        }

        if new_set.len() < self.set.len() {
            self.set.clear();
            for (_a, c) in new_set.iter() {
                self.set.push(c.clone());
            }
            self.set.sort_by_key(|c| c.a);
        }

        // now possibly shortening the selections.
        for i in (1..self.set.len()).rev() {
            let prev = self.set[i - 1].clone();
            let curr = &mut self.set[i];

            match (&mut curr.s, prev.s) {
                (Some(curr_s), Some(prev_s)) => {
                    // it's a little easier because I know from above sorts, that curr.a < next.a
                    if curr_s.b < prev_s.e {
                        curr_s.b = prev_s.e;
                        debug_assert!(curr.a == curr_s.e);
                    }
                }
                _ => {}
            }
        }

        debug_assert!(!self.set.is_empty());
    }

    pub fn word_begin<F: Fn(usize) -> bool>(&mut self, selecting: bool, word_determinant: &F) -> bool {
        let mut res = false;

        for c in self.set.iter_mut() {
            res |= c.word_begin(selecting, word_determinant);
        }

        self.reduce_left();

        res
    }

    pub fn word_end<F: FnMut(char) -> bool + Clone>(&mut self, buffer: &dyn TextBuffer, selecting: bool, word_determinant: &F) -> bool {
        let mut res = false;

        for c in self.set.iter_mut() {
            res |= c.word_end(buffer, selecting, word_determinant.clone());
        }

        self.reduce_right();

        res
    }

    pub fn word_begin_default(&mut self, buffer: &dyn TextBuffer, selecting: bool) -> bool {
        self.word_begin(
            selecting,
            &|idx: usize| -> bool {
                match buffer.char_at(idx) {
                    None => false,
                    Some(ch) => !ch.is_whitespace()
                }
            },
        )
    }

    pub fn word_end_default(&mut self, buffer: &dyn TextBuffer, selecting: bool) -> bool {
        self.word_end(
            buffer,
            selecting,
            &|c : char| -> bool {
                match self.

                match buffer.char_at(idx) {
                    None => false,
                    Some(ch) => !ch.is_whitespace()
                }
            },
        )
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

            debug_assert!(self.check_invariants());
            true
        } else {
            false
        }
    }

    pub fn check_invariants(&self) -> bool {
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
                error!("cursor[{}] = {:?} >= {:?} = cursor[{}]", idx - 1, self.set[idx-1], self.set[idx], idx);
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

        // at this point I know they are sorted and anchor-aligned. All I need to do is to check if begin is after previous end.
        for idx in 1..self.set.len() {
            if self.set[idx - 1].get_end() > self.set[idx].get_begin() {
                error!("cursor[{}].get_end() = {} > {} = cursor[{}].get_begin()", idx - 1, self.set[idx-1].get_end(), self.set[idx].get_begin(), idx);
                return false;
            }
        }

        true
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn remove_by_anchor(&mut self, anchor_char: usize) -> bool {
        for i in 0..self.set.len() {
            if self.set[i].a == anchor_char {
                self.set.remove(i);
                // disabled, we now allow "temporarily" empty sets in processing shift_tab
                //debug_assert!(self.check_invariants());
                return true;
            }
        }

        false
    }

    pub fn supercursor(&self) -> &Cursor {
        //TODO this should vary, depending on which direction cursors were moved last. Now it just points to first one.

        // this succeeds, because of invariants
        self.set.first().unwrap_or_else(|| {
            error!("invariant broken, empty cursor_set.");
            &ZERO_CURSOR
        })
    }

    pub fn is_single(&self) -> bool {
        self.set.len() == 1
    }
}
