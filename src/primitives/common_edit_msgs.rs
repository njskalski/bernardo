use std::collections::HashSet;
use std::ops::{BitOrAssign, Range};

use log::{error, warn};
use streaming_iterator::StreamingIterator;
use unicode_segmentation::UnicodeSegmentation;

use crate::app::App;
use crate::config::config::{CommonEditMsgKeybindings, ConfigRef};
use crate::cursor::cursor::Cursor;
use crate::cursor::cursor_set::CursorSet;
use crate::experiments::clipboard::ClipboardRef;
use crate::io::keys::{Key, Keycode};
use crate::primitives::arrow::Arrow;
use crate::primitives::has_invariant::HasInvariant;
use crate::primitives::printable::Printable;
use crate::text::text_buffer::TextBuffer;

/*
So I don't have to reimplement basic edit properties for multiple widgets, I moved all (cursor, content) related code here.
 */

// this is a completely arbitrary number against which I compare Page length, to avoid
// under/overflow while casting to isize safely.
const PAGE_HEIGHT_LIMIT: usize = 2000;

// TODO remove the integer result, it's not binding nor used anyways

/*
I am beginning to think, CEM should be broken into several smaller types, like:
move
edit
undo/redo
 */
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum CommonEditMsg {
    Char(char),
    Block(String),
    CursorUp { selecting: bool },
    CursorDown { selecting: bool },
    CursorLeft { selecting: bool },
    CursorRight { selecting: bool },
    Backspace,
    LineBegin { selecting: bool },
    LineEnd { selecting: bool },
    WordBegin { selecting: bool },
    WordEnd { selecting: bool },
    PageUp { selecting: bool },
    PageDown { selecting: bool },
    Delete,

    Copy,
    Paste,
    Undo,
    Redo,

    DeleteBlock { char_range: Range<usize> },
    InsertBlock { char_pos: usize, what: String },
    // not tested, created to remove _apply_stupid_substitute_message from BufferState
    SubstituteBlock { char_range: Range<usize>, with_what: String },
    Tab,
    ShiftTab,
}

impl CommonEditMsg {
    pub fn is_selecting(&self) -> bool {
        match self {
            CommonEditMsg::CursorUp { selecting } => *selecting,
            CommonEditMsg::CursorDown { selecting } => *selecting,
            CommonEditMsg::CursorLeft { selecting } => *selecting,
            CommonEditMsg::CursorRight { selecting } => *selecting,
            CommonEditMsg::LineBegin { selecting } => *selecting,
            CommonEditMsg::LineEnd { selecting } => *selecting,
            CommonEditMsg::WordBegin { selecting } => *selecting,
            CommonEditMsg::WordEnd { selecting } => *selecting,
            CommonEditMsg::PageUp { selecting } => *selecting,
            CommonEditMsg::PageDown { selecting } => *selecting,
            _ => false,
        }
    }

    pub fn without_selection(&self) -> Self {
        match self {
            CommonEditMsg::CursorUp { .. } => CommonEditMsg::CursorUp { selecting: false },
            CommonEditMsg::CursorDown { .. } => CommonEditMsg::CursorDown { selecting: false },
            CommonEditMsg::CursorLeft { .. } => CommonEditMsg::CursorLeft { selecting: false },
            CommonEditMsg::CursorRight { .. } => CommonEditMsg::CursorRight { selecting: false },
            CommonEditMsg::LineBegin { .. } => CommonEditMsg::LineBegin { selecting: false },
            CommonEditMsg::LineEnd { .. } => CommonEditMsg::LineEnd { selecting: false },
            CommonEditMsg::WordBegin { .. } => CommonEditMsg::WordBegin { selecting: false },
            CommonEditMsg::WordEnd { .. } => CommonEditMsg::WordEnd { selecting: false },
            CommonEditMsg::PageUp { .. } => CommonEditMsg::PageUp { selecting: false },
            CommonEditMsg::PageDown { .. } => CommonEditMsg::PageDown { selecting: false },
            x => x.clone(),
        }
    }

    pub fn is_editing(&self) -> bool {
        match self {
            CommonEditMsg::CursorUp { .. } => false,
            CommonEditMsg::CursorDown { .. } => false,
            CommonEditMsg::CursorLeft { .. } => false,
            CommonEditMsg::CursorRight { .. } => false,
            CommonEditMsg::LineBegin { .. } => false,
            CommonEditMsg::LineEnd { .. } => false,
            CommonEditMsg::WordBegin { .. } => false,
            CommonEditMsg::WordEnd { .. } => false,
            CommonEditMsg::PageUp { .. } => false,
            CommonEditMsg::PageDown { .. } => false,
            CommonEditMsg::Char(..) => true,
            CommonEditMsg::Block(..) => true,
            CommonEditMsg::Backspace => true,
            CommonEditMsg::Delete => true,
            CommonEditMsg::Copy => false,
            CommonEditMsg::Paste => true,
            CommonEditMsg::Undo => true,
            CommonEditMsg::Redo => true,
            CommonEditMsg::DeleteBlock { .. } => true,
            CommonEditMsg::InsertBlock { .. } => true,
            CommonEditMsg::SubstituteBlock { .. } => true,
            CommonEditMsg::Tab => true,
            CommonEditMsg::ShiftTab => true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ApplyCemResult {
    pub modified_buffer: bool,
    pub modified_cursor_set: bool,
    pub modified_other_cursor_set: bool,
}

impl BitOrAssign for ApplyCemResult {
    fn bitor_assign(&mut self, rhs: Self) {
        self.modified_buffer |= rhs.modified_buffer;
        self.modified_cursor_set |= rhs.modified_cursor_set;
        self.modified_other_cursor_set |= rhs.modified_other_cursor_set;
    }
}

impl ApplyCemResult {
    pub fn default() -> Self {
        Self {
            modified_buffer: false,
            modified_cursor_set: false,
            modified_other_cursor_set: false,
        }
    }

    pub fn or_with(&mut self, other: &ApplyCemResult) {
        self.modified_buffer |= other.modified_buffer;
        self.modified_cursor_set |= other.modified_buffer;
        self.modified_other_cursor_set |= other.modified_other_cursor_set;
    }
}

// This is where the mapping of keys to Msgs is
// TODO one needs to document here, that we treat Shift very specially
pub fn key_to_edit_msg(mut key: Key, keybindings: &CommonEditMsgKeybindings) -> Option<CommonEditMsg> {
    let modifiers = key.modifiers;

    if key != keybindings.shift_tab {
        key.modifiers.shift = false;
    }

    if key == keybindings.copy {
        return Some(CommonEditMsg::Copy);
    }
    if key == keybindings.paste {
        return Some(CommonEditMsg::Paste);
    }
    if key == keybindings.undo {
        return Some(CommonEditMsg::Undo);
    }
    if key == keybindings.redo {
        return Some(CommonEditMsg::Redo);
    }
    if key == keybindings.cursor_up {
        return Some(CommonEditMsg::CursorUp {
            selecting: modifiers.shift,
        });
    }
    if key == keybindings.cursor_down {
        return Some(CommonEditMsg::CursorDown {
            selecting: modifiers.shift,
        });
    }
    if key == keybindings.cursor_left {
        return Some(CommonEditMsg::CursorLeft {
            selecting: modifiers.shift,
        });
    }
    if key == keybindings.word_begin {
        return Some(CommonEditMsg::WordBegin {
            selecting: modifiers.shift,
        });
    }
    if key == keybindings.cursor_right {
        return Some(CommonEditMsg::CursorRight {
            selecting: modifiers.shift,
        });
    }
    if key == keybindings.word_end {
        return Some(CommonEditMsg::WordEnd {
            selecting: modifiers.shift,
        });
    }
    if key.keycode == Keycode::Enter {
        return Some(CommonEditMsg::Char('\n'));
    }
    if key.keycode == Keycode::Space {
        return Some(CommonEditMsg::Char(' '));
    }
    if key == keybindings.backspace {
        return Some(CommonEditMsg::Backspace);
    }
    if key == keybindings.line_begin || key.keycode == keybindings.home.keycode {
        return Some(CommonEditMsg::LineBegin {
            selecting: modifiers.shift,
        });
    }
    if key == keybindings.line_end {
        return Some(CommonEditMsg::LineEnd {
            selecting: modifiers.shift,
        });
    }
    if key == keybindings.page_up {
        return Some(CommonEditMsg::PageUp {
            selecting: modifiers.shift,
        });
    }
    if key == keybindings.page_down {
        return Some(CommonEditMsg::PageDown {
            selecting: modifiers.shift,
        });
    }
    if key == keybindings.tab {
        return Some(CommonEditMsg::Tab);
    }
    if key == keybindings.shift_tab {
        return Some(CommonEditMsg::ShiftTab);
    }
    if key == keybindings.delete {
        return Some(CommonEditMsg::Delete);
    }
    if let Keycode::Char(c) = key.keycode {
        if modifiers.is_empty() || modifiers.just_shift() {
            return Some(CommonEditMsg::Char(c));
        }
    }

    None
}

// returns sorted vector of line indices, reduced (no duplicates)
fn cursors_to_line_indices(rope: &dyn TextBuffer, cs: &CursorSet) -> Vec<usize> {
    let mut hs: HashSet<usize> = HashSet::new();
    for c in cs.iter() {
        rope.char_to_line(c.a)
            .map(|line_idx| {
                hs.insert(line_idx);
            })
            .unwrap_or_else(|| {
                error!("failed finding line for anchor {}", c.a);
            });

        c.s.map(|sel| {
            for char_idx in sel.b..sel.e {
                rope.char_to_line(char_idx)
                    .map(|line_idx| {
                        hs.insert(line_idx);
                    })
                    .unwrap_or_else(|| {
                        error!("failed finding line for selected index {}", char_idx);
                    })
            }
        });
    }

    let mut res: Vec<usize> = hs.into_iter().collect();
    res.sort();
    res
}

/*
returns whether any change happened (to text or cursors)
 */
// TODO assumptions that filelen < usize all over the place, assumption that diff < usize
fn insert_to_rope(
    cursor_set: &mut CursorSet,
    other_cursor_sets: &mut Vec<&mut CursorSet>,
    rope: &mut dyn TextBuffer,
    // if None, all cursors will input the same. If Some(idx) only this cursor will insert and rest will get updated.
    specific_cursor: Option<usize>,
    what: &str,
) -> ApplyCemResult {
    let mut res = ApplyCemResult::default();
    let mut modifier: isize = 0;
    for (cursor_idx, c) in cursor_set.iter_mut().enumerate() {
        res.modified_cursor_set |= c.shift_by(modifier);

        if cfg!(debug_assertions) {
            // equal is OK.
            if c.a > rope.len_chars() {
                error!("cursor beyond length of rope: {} > {}", c.a, rope.len_chars());
                continue;
            }
        }

        if specific_cursor.map(|idx| idx == cursor_idx).unwrap_or(true) {
            // whatever was selected, it's gone.
            if let Some(sel) = c.s {
                if rope.remove(sel.b, sel.e) {
                    res.modified_buffer |= true;
                    res.modified_cursor_set = true;
                } else {
                    warn!("expected to remove non-empty item substring but failed");
                }

                // TODO underflow/overflow
                let change = (sel.e - sel.b) as isize;
                modifier -= change;

                for other_cursor_set in other_cursor_sets.iter_mut() {
                    res.modified_other_cursor_set |= update_cursors_after_removal(other_cursor_set, sel.b..sel.e);
                }

                // this is necessary, because otherwise shift below may fail. I copy out, since later there is no
                // anchor.
                let was_anchor_right = c.anchor_right();
                c.clear_selection();

                if was_anchor_right {
                    res.modified_cursor_set |= c.shift_by(-change);
                }
            }
            c.clear_pc();

            let what_len = what.graphemes(true).count();
            if rope.insert_block(c.a, what) {
                res.modified_buffer |= true;
                res.modified_cursor_set = true;

                let stride = what.graphemes(true).count();
                for other_cursor_set in other_cursor_sets.iter_mut() {
                    res.modified_other_cursor_set |= update_cursors_after_insertion(other_cursor_set, c.a, stride);
                }
            } else {
                warn!("expected to insert {} characters at {}, but failed", what_len, c.a);
            }

            res.modified_cursor_set |= c.shift_by(what_len as isize);
            modifier += what_len as isize;

            debug_assert!(c.check_invariant());
        }
    }
    res.modified_cursor_set |= cursor_set.reduce_right();
    debug_assert!(cursor_set.check_invariant());
    res
}

// returns whether rope has changed
fn insert_to_rope_at_random_place(
    cursor_set: &mut CursorSet,
    other_cursor_sets: &mut Vec<&mut CursorSet>,
    rope: &mut dyn TextBuffer,
    char_pos: usize,
    what: &str,
) -> ApplyCemResult {
    let mut res = ApplyCemResult::default();
    if !rope.insert_block(char_pos, &what) {
        error!("did not insert to rope at {}", char_pos);
    } else {
        res.modified_buffer = true;
        let stride = what.graphemes(true).count();

        res.modified_cursor_set |= update_cursors_after_insertion(cursor_set, char_pos, stride);

        for other_cursor_set in other_cursor_sets.iter_mut() {
            res.modified_other_cursor_set |= update_cursors_after_insertion(other_cursor_set, char_pos, stride);
        }
    }
    res
}

fn update_cursors_after_insertion(cs: &mut CursorSet, char_pos: usize, char_len: usize) -> bool {
    let mut res = false;
    for c in cs.iter_mut() {
        if char_pos <= c.get_begin() {
            res |= c.shift_by(char_len as isize); // TODO overflow
        } else {
            if char_pos < c.get_end() {
                if let Some(sel) = c.s.as_mut() {
                    if c.a == sel.e {
                        c.a += char_len;
                        sel.e += char_len;
                    } else {
                        sel.e += char_len;
                    }
                    res = true;
                }
            }
        }
    }

    debug_assert!(cs.check_invariant());
    res
}

// Returns true if CursorSet was modified
fn update_cursors_after_removal(cs: &mut CursorSet, char_range: Range<usize>) -> bool {
    // first, throwing away cursors inside the block
    let mut res = false;
    {
        let mut to_remove: Vec<usize> = Vec::new();
        for c in cs.iter() {
            // the first ineq must be sharp - removing char that would have been *replaced* by cursor, does not
            // invalidate the cursor
            if char_range.start < c.get_begin() && c.get_end() <= char_range.end {
                to_remove.push(c.a);
            }
        }

        for ca in to_remove.into_iter() {
            let paranoia = cs.remove_by_anchor(ca);
            debug_assert!(paranoia);
            res |= paranoia;
        }
    }
    let stride = char_range.len();
    // second, cutting cursors overlapping with block
    {
        for c in cs.iter_mut() {
            if !c.is_simple() && c.intersects(&char_range) {
                // end inside
                if char_range.contains(&c.get_end()) {
                    debug_assert!(char_range.contains(&c.s.unwrap().e));

                    if let Some(sel) = c.s.as_mut() {
                        if c.a == sel.e {
                            sel.e = char_range.start;
                            c.a = char_range.start;
                        } else {
                            sel.e = char_range.start;
                        }
                        res |= true;
                    }
                    continue;
                }

                // begin inside
                if char_range.contains(&c.a) {
                    if let Some(sel) = c.s.as_mut() {
                        if c.a == sel.b {
                            sel.b = char_range.end;
                            c.a = char_range.end;
                        } else {
                            sel.b = char_range.end;
                        }
                        res |= true;
                    }
                    continue;
                }

                // entire block was inside cursor
                if let Some(sel) = c.s.as_mut() {
                    debug_assert!(sel.b <= char_range.start);
                    debug_assert!(char_range.end <= sel.e);
                    debug_assert!(sel.e - sel.b >= char_range.len());

                    if c.a == sel.e {
                        c.a -= stride;
                        sel.e -= stride;
                    } else {
                        sel.e -= stride;
                    }
                    res |= true;
                    continue;
                }

                debug_assert!(false);
            }
        }
    }

    // moving all cursors that were after the removed block
    for c in cs.iter_mut() {
        if char_range.end <= c.get_begin() {
            res |= c.shift_by(-(stride as isize)); // TODO overflow
        }
    }

    if cs.len() == 0 {
        error!("cs empty after removing block, will set to a single cursor at block start");
        res |= cs.add_cursor(Cursor::new(char_range.start));
    }
    res
}

// returns whether a change occurred
fn remove_from_rope_at_random_place(
    cursor_set: &mut CursorSet,
    other_cursor_sets: &mut Vec<&mut CursorSet>,
    rope: &mut dyn TextBuffer,
    char_range: Range<usize>,
) -> ApplyCemResult {
    let mut res = ApplyCemResult::default();
    if char_range.is_empty() {
        error!("delete block with empty range, ignoring");
    } else if !rope.remove(char_range.start, char_range.end) {
        error!("failed to remove block");
    } else {
        res.modified_buffer = true;
        res.modified_cursor_set |= update_cursors_after_removal(cursor_set, char_range.clone());

        for other_cursor_set in other_cursor_sets.iter_mut() {
            res.modified_other_cursor_set |= update_cursors_after_removal(other_cursor_set, char_range.clone());
        }
    }
    res
}

// returns whether change occured
fn handle_backspace_and_delete(
    cursor_set: &mut CursorSet,
    other_cursor_sets: &mut Vec<&mut CursorSet>,
    backspace: bool,
    rope: &mut dyn TextBuffer,
) -> ApplyCemResult {
    let mut res = ApplyCemResult::default();
    let mut modifier: isize = 0;
    for c in cursor_set.iter_mut() {
        res.modified_cursor_set |= c.shift_by(modifier);

        if cfg!(debug_assertions) {
            // equal is OK.
            if c.a > rope.len_chars() {
                error!("cursor beyond length of rope: {} > {}", c.a, rope.len_chars());
                continue;
            }
        }

        if let Some(sel) = c.s {
            if rope.remove(sel.b, sel.e) {
                res.modified_buffer |= true;

                for other_cursor_set in other_cursor_sets.iter_mut() {
                    res.modified_other_cursor_set |= update_cursors_after_removal(other_cursor_set, sel.b..sel.e);
                }
            } else {
                warn!("expected to remove non-empty item substring but failed");
            }

            // TODO underflow/overflow
            let change = (sel.e - sel.b) as isize;
            modifier -= change;

            let anchor_right = c.anchor_right();
            res.modified_cursor_set |= c.clear_both();

            if anchor_right {
                res.modified_cursor_set |= c.shift_by(-change);
            }
        } else {
            if backspace {
                if c.a == 0 {
                    continue;
                }
            } else if c.a == rope.len_chars() {
                continue;
            }

            let (b, e) = if backspace { (c.a - 1, c.a) } else { (c.a, c.a + 1) };

            if rope.remove(b, e) {
                res.modified_buffer |= true;

                for other_cursor_set in other_cursor_sets.iter_mut() {
                    res.modified_other_cursor_set |= update_cursors_after_removal(other_cursor_set, b..e);
                }
            } else {
                error!("expected to remove char but failed");
            }
            modifier -= 1;

            res.modified_cursor_set |= c.clear_both();

            if backspace {
                res.modified_cursor_set |= c.shift_by(-1);
            }
        }

        debug_assert!(c.check_invariant());
    }

    cursor_set.reduce_left();
    debug_assert!(cursor_set.check_invariant());
    res
}

/*
Returns references to modified buffer or None and ref to modified CursorSet or None


3) observer_cursor_sets - these will be forcibly updated by primary cursor set. Why?
    - because i) we don't really care that much, it's a rare use case
              ii) we can't really "store them" for undo/redo, because observer (as name suggests)
                  is not the same entity as primary editor. If we "jumped" observer cursors on
                  undo/redo, we would "interrupt flow" of observer. Destroying an invalid cursor
                  is... less destructive than destroying flow.
 */
pub fn apply_common_edit_message(
    cem: CommonEditMsg,
    cursor_set: &mut CursorSet,
    observer_cursor_sets: &mut Vec<&mut CursorSet>,
    rope: &mut dyn TextBuffer,
    page_height: usize,
    clipboard: Option<&ClipboardRef>,
    tabs_to_spaces: Option<u8>,
) -> ApplyCemResult {
    let mut res = ApplyCemResult::default();
    match cem {
        CommonEditMsg::Char(char) => {
            // TODO optimise
            {
                res |= insert_to_rope(cursor_set, observer_cursor_sets, rope, None, char.to_string().as_str());
            }
        }
        CommonEditMsg::Block(s) => {
            res |= insert_to_rope(cursor_set, observer_cursor_sets, rope, None, &s);
        }
        CommonEditMsg::CursorUp { selecting } => {
            res.modified_cursor_set |= cursor_set.move_vertically_by(rope, -1, selecting);
        }
        CommonEditMsg::CursorDown { selecting } => {
            res.modified_cursor_set |= cursor_set.move_vertically_by(rope, 1, selecting);
        }
        CommonEditMsg::CursorLeft { selecting } => {
            res.modified_cursor_set |= cursor_set.move_left(selecting);
        }
        CommonEditMsg::CursorRight { selecting } => {
            res.modified_cursor_set |= cursor_set.move_right(rope, selecting);
        }
        CommonEditMsg::Backspace => {
            res |= handle_backspace_and_delete(cursor_set, observer_cursor_sets, true, rope);
        }
        CommonEditMsg::LineBegin { selecting } => {
            res.modified_cursor_set |= cursor_set.move_home(rope, selecting);
        }
        CommonEditMsg::LineEnd { selecting } => {
            res.modified_cursor_set |= cursor_set.move_end(rope, selecting);
        }
        CommonEditMsg::WordBegin { selecting } => {
            res.modified_cursor_set |= cursor_set.word_begin_default(rope, selecting);
        }
        CommonEditMsg::WordEnd { selecting } => {
            res.modified_cursor_set |= cursor_set.word_end_default(rope, selecting);
        }
        CommonEditMsg::PageUp { selecting } => {
            if page_height > PAGE_HEIGHT_LIMIT {
                error!("received PageUp of page_height {}, ignoring.", page_height);
            } else {
                res.modified_cursor_set |= cursor_set.move_vertically_by(rope, -(page_height as isize), selecting);
            }
        }
        CommonEditMsg::PageDown { selecting } => {
            if page_height > PAGE_HEIGHT_LIMIT {
                error!("received PageDown of page_height {}, ignoring.", page_height);
            } else {
                res.modified_cursor_set |= cursor_set.move_vertically_by(rope, page_height as isize, selecting);
            }
        }
        CommonEditMsg::Delete => res |= handle_backspace_and_delete(cursor_set, observer_cursor_sets, false, rope),
        CommonEditMsg::Copy => {
            if let Some(clipboard) = clipboard {
                let mut contents = String::new();
                for c in cursor_set.iter() {
                    if !contents.is_empty() {
                        contents.push('\n');
                    }

                    if let Some(sel) = c.s {
                        for c in rope.chars().skip(sel.b).take(sel.e - sel.b) {
                            contents.push(c);
                        }
                    }
                }

                clipboard.set(contents);
            } else {
                warn!("copy without a clipboard, ignoring");
            }
        }
        CommonEditMsg::Paste => {
            if let Some(clipboard) = clipboard {
                let cursor_count = cursor_set.set().len();
                let contents = clipboard.get();
                if contents.is_empty() {
                    warn!("not pasting empty contents");
                } else {
                    let split_lines = contents.lines().count() == cursor_count;
                    // easy, each cursor gets full copy
                    if !split_lines {
                        res |= insert_to_rope(cursor_set, observer_cursor_sets, rope, None, &contents);
                    } else {
                        let mut it = contents.lines();
                        let mut common_idx: usize = 0;
                        while let Some(line) = it.next() {
                            res |= insert_to_rope(cursor_set, observer_cursor_sets, rope, Some(common_idx), line);
                            common_idx += 1;
                        }
                    }
                }
            } else {
                warn!("paste without a clipboard, ignoring");
            }
        }
        CommonEditMsg::Undo => res.modified_buffer |= rope.undo(),
        CommonEditMsg::Redo => res.modified_buffer |= rope.redo(),
        CommonEditMsg::DeleteBlock { char_range } => {
            res |= remove_from_rope_at_random_place(cursor_set, observer_cursor_sets, rope, char_range)
        }
        CommonEditMsg::InsertBlock { char_pos, what } => {
            res |= insert_to_rope_at_random_place(cursor_set, observer_cursor_sets, rope, char_pos, &what);
        }
        CommonEditMsg::Tab => {
            let tab = match tabs_to_spaces {
                None => "\t".to_string(),
                Some(i) => {
                    let mut tab: String = String::new();
                    for _ in 0..i {
                        tab.push(' ');
                    }
                    tab
                }
            };

            // if they are simple, we just add spaces
            if cursor_set.are_simple() {
                res |= insert_to_rope(cursor_set, observer_cursor_sets, rope, None, &tab);
            } else {
                let all_complex = cursor_set.iter().fold(true, |acc, c| acc && !c.is_simple());
                if !all_complex {
                    error!("ignoring tab on mixed cursor set");
                } else {
                    let indices = cursors_to_line_indices(rope, cursor_set);
                    for line_idx in indices.into_iter() {
                        if let Some(char_begin_idx) = rope.line_to_char(line_idx) {
                            res |= insert_to_rope_at_random_place(cursor_set, observer_cursor_sets, rope, char_begin_idx, &tab);
                        } else {
                            error!("failed casting line_idx to begin char (1)");
                        }
                    }
                }
            }
        }
        CommonEditMsg::ShiftTab => {
            let indices = cursors_to_line_indices(rope, cursor_set);

            for line_idx in indices.iter().rev() {
                if let Some(char_begin_idx) = rope.line_to_char(*line_idx) {
                    let mut how_many_chars_to_eat: usize = 0;
                    let charat = match rope.char_at(char_begin_idx) {
                        Some(c) => c,
                        None => {
                            error!("no character at char_begin_idx {}", char_begin_idx);
                            continue;
                        }
                    };
                    // debug!("line {} char {} : [{}]", *line_idx, char_begin_idx, charat);

                    if charat == '\t' {
                        how_many_chars_to_eat = 1;
                    } else {
                        let tab_width = match tabs_to_spaces {
                            None => 1,
                            Some(i) => i,
                        } as usize;

                        'dig_prefix: for offset in 0..tab_width {
                            // I ignore the '\t' characters.
                            let new_charat = match rope.char_at(char_begin_idx + offset) {
                                Some(c) => c,
                                None => {
                                    error!("no character at char_begin_idx + offset {}", char_begin_idx + offset);
                                    continue;
                                }
                            };
                            // debug!("line {} char2 {} : [{}]", *line_idx, char_begin_idx, new_charat);
                            if new_charat == ' ' {
                                how_many_chars_to_eat += 1;
                            } else {
                                break 'dig_prefix;
                            }
                        }
                    }

                    if how_many_chars_to_eat == 0 {
                        continue;
                    }

                    // debug!("ordering to remove from line {} [{}] chars", line_idx, how_many_chars_to_eat);
                    let partial_res = remove_from_rope_at_random_place(
                        cursor_set,
                        observer_cursor_sets,
                        rope,
                        char_begin_idx..char_begin_idx + how_many_chars_to_eat,
                    );

                    res |= partial_res;
                } else {
                    error!("failed casting line_idx to begin char (2)");
                }
            }
        }
        CommonEditMsg::SubstituteBlock { char_range, with_what } => {
            if !char_range.is_empty() {
                res |= remove_from_rope_at_random_place(cursor_set, observer_cursor_sets, rope, char_range.clone());
            }

            if res.modified_buffer {
                res |= insert_to_rope_at_random_place(cursor_set, observer_cursor_sets, rope, char_range.start, &with_what);
            }
        }
    };

    debug_assert!(cursor_set.check_invariant());

    for c in cursor_set.iter() {
        debug_assert!(c.get_end() <= rope.len_chars());
    }

    res
}

// This maps a cme into a direction that the cursor has (probably) moved. It's used to update
// scrolling information.
// TODO Undo/Redo should update cursor!
pub fn cme_to_direction(cme: &CommonEditMsg) -> Option<Arrow> {
    match cme {
        CommonEditMsg::Char(_) => Some(Arrow::Right),
        CommonEditMsg::Block(_) => Some(Arrow::Right),
        CommonEditMsg::CursorUp { .. } => Some(Arrow::Up),
        CommonEditMsg::CursorDown { .. } => Some(Arrow::Down),
        CommonEditMsg::CursorLeft { .. } => Some(Arrow::Left),
        CommonEditMsg::CursorRight { .. } => Some(Arrow::Right),
        CommonEditMsg::Backspace => Some(Arrow::Left),
        CommonEditMsg::LineBegin { .. } => Some(Arrow::Left),
        CommonEditMsg::LineEnd { .. } => Some(Arrow::Right),
        CommonEditMsg::WordBegin { .. } => Some(Arrow::Left),
        CommonEditMsg::WordEnd { .. } => Some(Arrow::Right),
        CommonEditMsg::PageUp { .. } => Some(Arrow::Up),
        CommonEditMsg::PageDown { .. } => Some(Arrow::Down),
        CommonEditMsg::Delete => None,
        CommonEditMsg::Copy => None,
        CommonEditMsg::Paste => Some(Arrow::Right),
        CommonEditMsg::Undo => None,
        CommonEditMsg::Redo => None,
        CommonEditMsg::ShiftTab => Some(Arrow::Left),
        CommonEditMsg::Tab => Some(Arrow::Right),
        // TODO these 3 can be implemented. Do I want it?
        CommonEditMsg::DeleteBlock { .. } => None,
        CommonEditMsg::InsertBlock { .. } => None,
        CommonEditMsg::SubstituteBlock { .. } => None,
    }
}
