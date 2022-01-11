// Copyright 2018-2020 Google LLC, 2021 Andrzej J Skalski
// This version of the file (2021+) is licensed with GNU LGPLv3 License.
// For older version of file (licensed under Apache 2 license), see sly-editor, at
// https://github.com/njskalski/sly-editor/blob/master/src/cursor_set.rs

// Cursor == (Selection, Anchor), thanks Kakoune!
// both positions and anchor are counted in CHARS not offsets.

// The cursor points to a index where a NEW character will be included, or old character will be
// REPLACED.

// Cursor pointing to a newline character is visualized as an option to append preceding it line.

// So Cursor can point 1 character BEYOND length of buffer!

// Newline is always an end of previous line, not a beginning of new.


use std::collections::{HashMap, HashSet};
use std::slice::Iter;



use crate::text::buffer::Buffer;

const NEWLINE_LENGTH: usize = 1; // TODO(njskalski): add support for multisymbol newlines?

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CursorStatus {
    None,
    WithinSelection,
    UnderCursor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Selection {
    pub b: usize,
    //begin inclusive
    pub e: usize, //end EXCLUSIVE (as *everywhere*)
}

impl Selection {
    pub fn new(b: usize, e: usize) -> Self {
        debug_assert!(b < e);
        Selection {
            b,
            e,
        }
    }

    pub fn within(self, char_idx: usize) -> bool {
        char_idx >= self.b && char_idx < self.e
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cursor {
    pub s: Option<Selection>,
    // selection
    pub a: usize,
    //anchor
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
    pub fn home(&mut self, rope: &dyn Buffer, selecting: bool) -> bool {
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
                self.s = Some(Selection::new(new_pos, old_pos));
            }
            self.preferred_column = None;

            true
        };

        res
    }

    // Returns FALSE if noop.
    pub fn end(&mut self, rope: &dyn Buffer, selecting: bool) -> bool {
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
                self.s = Some(Selection::new(old_pos, new_pos))
            }
            self.preferred_column = None;

            true
        };

        res
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

    pub fn set(&self) -> &Vec<Cursor> {
        &self.set
    }

    // Returns only element OR None if the set is NOT a singleton.
    pub fn as_single(&self) -> Option<&Cursor> {
        if self.set.len() != 1 {
            None
        } else {
            self.set.first()
        }
    }
}

impl CursorSet {
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
                    c.s = match c.s {
                        None => Some(Selection::new(c.a, old_pos)),
                        Some(old_sel) => Some(Selection::new(c.a, old_sel.e)),
                    };
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

        res
    }

    pub fn move_right(&mut self, rope: &dyn Buffer, selecting: bool) -> bool {
        self.move_right_by(rope, 1, selecting)
    }

    pub fn move_right_by(&mut self, rope: &dyn Buffer, l: usize, selecting: bool) -> bool {
        let len = rope.len_chars();
        let mut res = false;

        for mut c in &mut self.set {
            //we allow anchor after last char (so you can backspace last char)
            if c.a < len {
                let old_pos = c.a;
                c.a = std::cmp::min(c.a + l, len);

                if selecting {
                    c.s = match c.s {
                        None => Some(Selection::new(old_pos, c.a)),
                        Some(old_sel) => Some(Selection::new(old_sel.b, c.a)),
                    }
                } else {
                    c.clear_both();
                }

                res = true;
            };
        }

        // TODO test
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

        res
    }

    pub fn move_vertically_by(&mut self, rope: &dyn Buffer, l: isize, _selecting: bool) -> bool {
        if l == 0 {
            return false;
        }

        let last_line_idx = rope.len_lines() - 1;

        for mut c in &mut self.set {
            //getting data

            let cur_line_idx = match rope.char_to_line(c.a) {
                Some(line) => line,
                None => {
                    // now this gets fuzzy. So it's simple to
                    0 //TODO
                }
            };

            let cur_line_begin_char_idx = rope.line_to_char(cur_line_idx).unwrap(); //TODO
            let current_char_idx = c.a - cur_line_begin_char_idx;

            if cur_line_idx as isize + l > last_line_idx as isize
            /* && l > 0, checked before */
            {
                c.preferred_column = Some(current_char_idx);
                c.a = rope.len_chars(); // pointing to index higher than last valid one.
                continue;
            }

            if cur_line_idx as isize + l < 0 {
                c.preferred_column = Some(current_char_idx);
                c.a = 0;
                continue;
            }

            // at this point we know that 0 <= cur_line_idx <= last_line_idx
            debug_assert!(0 <= cur_line_idx);
            debug_assert!(cur_line_idx <= last_line_idx);
            let new_line_idx = (cur_line_idx as isize + l) as usize;

            // This is actually right. Ropey counts '\n' as last character of current line.
            let last_char_idx_in_new_line = if new_line_idx == last_line_idx {
                //this corresponds to a notion of "potential new character" beyond the buffer. It's a valid cursor position.
                rope.len_chars()
            } else {
                rope.line_to_char(new_line_idx + 1).unwrap() - NEWLINE_LENGTH //TODO
            };

            let new_line_begin = rope.line_to_char(new_line_idx).unwrap(); //TODO
            let new_line_num_chars = last_char_idx_in_new_line + 1 - new_line_begin;

            //setting data

            c.clear_selection();

            if let Some(preferred_column) = c.preferred_column {
                debug_assert!(preferred_column >= current_char_idx);
                if preferred_column <= new_line_num_chars {
                    c.clear_pc();
                    c.a = new_line_begin + preferred_column;
                } else {
                    c.a = new_line_begin + new_line_num_chars;
                }
            } else {
                let addon = if new_line_idx == last_line_idx { 1 } else { 0 };
                // inequality below is interesting.
                // The line with characters 012 is 3 characters long. So if current char idx is 3
                // it means that line below needs at least 4 character to host it without shift left.
                // "addon" is there to make sure that last line is counted as "one character longer"
                // than it actually is, so we can position cursor one character behind buffer
                // (appending).
                if new_line_num_chars + addon <= current_char_idx {
                    c.a = new_line_begin + new_line_num_chars - 1; //this -1 is needed.
                    c.preferred_column = Some(current_char_idx);
                } else {
                    c.a = new_line_begin + current_char_idx;
                }
            }
        }

        false
    }

    /// TODO(njskalski): how to reduce selections? Overlapping selections?
    /// TODO(njskalski): it would make a sense not to reduce cursors that have identical .a but different .preferred_column.
    /// Yet we want not to put characters twice for overlapping cursors.
    pub fn reduce(&mut self) {
        let _curs: HashSet<usize> = HashSet::new();

        //        dbg!(&self.set);

        let mut old_curs: Vec<Cursor> = vec![];
        std::mem::swap(&mut old_curs, &mut self.set);

        for c in &old_curs {
            let mut found = false;
            for oc in &self.set {
                if c.a == oc.a {
                    found = true;
                    break;
                }
            }

            if !found {
                self.set.push(c.clone());
            }
        }
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

    // Returns FALSE if results in no-op
    // TODO test
    // TODO there should be a reduction of overlapping. This case is easy - just pick the biggest.
    pub fn home(&mut self, rope: &dyn Buffer, selecting: bool) -> bool {
        let mut res = false;

        for c in self.set.iter_mut() {
            res |= c.home(rope, selecting);
        };

        // reducing - we just pick ones that are furthest right
        let mut new_set = HashMap::<usize, Cursor>::new();
        for c in self.set.iter() {
            match new_set.get(&c.a) {
                None => { new_set.insert(c.a, c.clone()); },
                Some(old_c) => {
                    // we replace only if old one has shorter selection than new one.
                    match (old_c.s, c.s) {
                        (Some(old_sel), Some(new_sel)) => {
                            if old_sel.e < new_sel.e {
                                new_set.insert(c.a, c.clone());
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        debug_assert!(new_set.len() <= self.set.len()); //midnight paranoia
        if new_set.len() < self.set.len() {
            self.set.clear();

            for (_a, c) in new_set.iter() {
                self.set.push(c.clone());
            }

            self.set.sort_by_key(|c| c.a);
        }
        res
    }

    // Returns FALSE if results in noop.
    pub fn end(&mut self, rope: &dyn Buffer, selecting: bool) -> bool {
        let mut res = false;

        for c in self.set.iter_mut() {
            res |= c.end(rope, selecting);
        };

        // reducing - we just pick ones that are furthest left
        let mut new_set = HashMap::<usize, Cursor>::new();
        for c in self.set.iter() {
            match new_set.get(&c.a) {
                None => { new_set.insert(c.a, c.clone()); },
                Some(old_c) => {
                    // we replace only if old one has shorter selection than new one.
                    match (old_c.s, c.s) {
                        (Some(old_sel), Some(new_sel)) => {
                            if new_sel.b < old_sel.b {
                                new_set.insert(c.a, c.clone());
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        debug_assert!(new_set.len() <= self.set.len()); //midnight paranoia
        if new_set.len() < self.set.len() {
            self.set.clear();

            for (_a, c) in new_set.iter() {
                self.set.push(c.clone());
            }

            self.set.sort_by_key(|c| c.a);
        }

        res
    }

    pub fn backspace(&mut self, rope: &mut dyn Buffer) -> bool {
        let mut res = false;

        // this has to be reverse iterator, otherwise the indices are messed up.
        for c in self.set.iter_mut().rev() {
            let (b, e) = match c.s {
                None => {
                    if c.a == 0 {
                        continue
                    };

                    (c.a - 1, c.a)
                }
                Some(sel) => (sel.b, sel.e),
            };

            res |= rope.remove(b, e);

            c.clear_both();
            c.a = b;
        }

        self.reduce();

        res
    }
}
