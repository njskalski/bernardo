// Copyright 2018-2020 Google LLC, 2021-2023 Andrzej J Skalski
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

use crate::primitives::has_invariant::HasInvariant;
use crate::text::text_buffer::TextBuffer;

pub const NEWLINE_LENGTH: usize = 1; // TODO(njskalski): add support for multisymbol newlines?

pub const ZERO_CURSOR: Cursor = Cursor::new(0);

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

/* both signatures are buffer, first idx, current idx, returns whether to continue moving cursor or not
TODO add what happens on any of indices being invalid. Right now I just return false, meaning "stop progressing"
 */
pub type ForwardWordDeterminant = dyn Fn(&dyn TextBuffer, usize, usize) -> bool;
pub type BackwardWordDeterminant = dyn Fn(&dyn TextBuffer, usize, usize) -> bool;

pub fn default_word_determinant(buffer: &dyn TextBuffer, first_idx: usize, current_idx: usize) -> bool {
    let return_value = match (buffer.char_at(first_idx), buffer.char_at(current_idx)) {
        (Some(first_char), Some(current_char)) => {
            first_char.is_whitespace() == current_char.is_whitespace()
        }
        _ => false,
    };

    // warn!("word {} first char {:?} curr_char {:?} wd {:?}",
    //             buffer.to_string(),
    //             buffer.char_at(first_idx),
    //             buffer.char_at(current_idx),
    //             return_value,
    //         );

    return_value
}

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
    pub(crate) fn word_begin(&mut self, buffer: &dyn TextBuffer, selecting: bool, word_determinant: &BackwardWordDeterminant) -> bool {
        if self.a == 0 {
            return false;
        }

        let old_pos = self.a;

        if self.a > 0 {
            self.a -= 1;

            // this is different than in word_end, because we want "more of the same as the first
            // character we jumped over", so we first move, then remember "what we jumped over"
            let first_char_pos = self.a;

            // if word_determinant(buffer, old_pos, self.a - 1) {
            // case when cursor is within a word
            while self.a > 0 && word_determinant(buffer, first_char_pos, self.a - 1) {
                self.a -= 1;
            }
            // }
        }


        if selecting {
            self.update_select(old_pos, self.a);
        } else {
            self.clear_selection();
        }

        debug_assert!(old_pos >= self.a);

        old_pos != self.a
    }

    pub(crate) fn word_end(&mut self, buffer: &dyn TextBuffer, selecting: bool, word_determinant: &ForwardWordDeterminant) -> bool {
        if self.a == buffer.len_chars() {
            return false;
        }

        let old_pos = self.a;

        if self.a < buffer.len_chars() {
            if word_determinant(buffer, old_pos, self.a) {
                // variant within the word
                while self.a < buffer.len_chars() && word_determinant(buffer, old_pos, self.a) {
                    self.a += 1;
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

impl HasInvariant for Cursor {
    fn check_invariant(&self) -> bool {
        if let Some(s) = self.s {
            s.b != s.e && (s.b == self.a || s.e == self.a)
        } else {
            true
        }
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