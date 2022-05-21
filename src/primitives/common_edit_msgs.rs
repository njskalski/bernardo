use log::{debug, error, warn};
use crate::experiments::clipboard::ClipboardRef;

use crate::io::keys::Key;
use crate::Keycode;
use crate::primitives::arrow::Arrow;
use crate::primitives::cursor_set::CursorSet;
use crate::text::buffer::Buffer;

/*
So I don't have to reimplement basic edit properties for multiple widgets, I moved all (cursor, content) related code here.
 */

// this is a completely arbitrary number against which I compare Page length, to avoid under/overflow while casting to isize safely.
const PAGE_HEIGHT_LIMIT: usize = 2000;

/*
I am beginning to think, CEM should be broken into several smaller types, like:
move
edit
undo/redo
 */
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CommonEditMsg {
    Char(char),
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
            x => *x,
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
            CommonEditMsg::Backspace => true,
            CommonEditMsg::Delete => true,
            CommonEditMsg::Copy => false,
            CommonEditMsg::Paste => true,
            CommonEditMsg::Undo => true,
            CommonEditMsg::Redo => true,
        }
    }
}

// This is where the mapping of keys to Msgs is
pub fn key_to_edit_msg(key: Key) -> Option<CommonEditMsg> {
    match key {
        Key { keycode, modifiers } => {
            match keycode {
                Keycode::Char('c') if modifiers.just_ctrl() => Some(CommonEditMsg::Copy),
                Keycode::Char('v') if modifiers.just_ctrl() => Some(CommonEditMsg::Paste),
                Keycode::Char('z') if modifiers.just_ctrl() => Some(CommonEditMsg::Undo),
                Keycode::Char('x') if modifiers.just_ctrl() => Some(CommonEditMsg::Redo),
                Keycode::Char(c) if modifiers.is_empty() => Some(CommonEditMsg::Char(c)),
                Keycode::ArrowUp => Some(CommonEditMsg::CursorUp { selecting: key.modifiers.shift }),
                Keycode::ArrowDown => Some(CommonEditMsg::CursorDown { selecting: key.modifiers.shift }),
                Keycode::ArrowLeft => {
                    if key.modifiers.ctrl {
                        Some(CommonEditMsg::WordBegin { selecting: key.modifiers.shift })
                    } else {
                        Some(CommonEditMsg::CursorLeft { selecting: key.modifiers.shift })
                    }
                }
                Keycode::ArrowRight => {
                    if key.modifiers.ctrl {
                        Some(CommonEditMsg::WordEnd { selecting: key.modifiers.shift })
                    } else {
                        Some(CommonEditMsg::CursorRight { selecting: key.modifiers.shift })
                    }
                }
                Keycode::Enter => {
                    // debug!("mapping Keycode:Enter to Char('\\n')");
                    Some(CommonEditMsg::Char('\n'))
                }
                Keycode::Space => {
                    // debug!("mapping Keycode:Space to Char(' ')");
                    Some(CommonEditMsg::Char(' '))
                }
                Keycode::Backspace => Some(CommonEditMsg::Backspace),
                Keycode::Home => Some(CommonEditMsg::LineBegin { selecting: key.modifiers.shift }),
                Keycode::End => Some(CommonEditMsg::LineEnd { selecting: key.modifiers.shift }),
                Keycode::PageUp => Some(CommonEditMsg::PageUp { selecting: key.modifiers.shift }),
                Keycode::PageDown => Some(CommonEditMsg::PageDown { selecting: key.modifiers.shift }),
                Keycode::Tab => {
                    // debug!("mapping Keycode:Tab to Char('\\t')");
                    Some(CommonEditMsg::Char('\t'))
                }
                Keycode::Delete => Some(CommonEditMsg::Delete),
                _ => None
            }
        }
    }
}

/*
returns tuple
    char changed (I aim at "Levenstein distance", but I have not tested that)
    whether any change happened (to text or cursors)
 */
// TODO assumptions that filelen < usize all over the place, assumption that diff < usize
fn insert_to_rope(cs: &mut CursorSet,
                  rope: &mut dyn Buffer,
                  // if None, all cursors will input the same. If Some(idx) only this cursor will insert and rest will get updated.
                  specific_cursor: Option<usize>,
                  what: &str) -> (usize, bool) {
    let mut res = false;
    let mut modifier: isize = 0;
    let mut diff_len: usize = 0;
    for (cursor_idx, c) in cs.iter_mut().enumerate() {
        let mut cursor_specific_diff_len: usize = 0;

        c.shift_by(modifier);

        if cfg!(debug_assertions) {
            // equal is OK.
            if c.a > rope.len_chars() {
                error!("cursor beyond length of rope: {} > {}", c.a, rope.len_chars());
                continue;
            }
        }

        if specific_cursor.map(|idx| idx == cursor_idx).unwrap_or(true) {
            if rope.insert_block(c.a, what) {
                res |= true;
                cursor_specific_diff_len = what.len();
            } else {
                warn!("expected to insert {} characters at {}, but failed", what.len(), c.a);
            }

            c.shift_by(what.len() as isize);
            modifier += what.len() as isize;

            // whatever was selected, it's gone.
            if let Some(sel) = c.s {
                if rope.remove(sel.b, sel.e) {
                    res |= true;
                    cursor_specific_diff_len = std::cmp::max(cursor_specific_diff_len, sel.len());
                } else {
                    warn!("expected to remove non-empty item substring but failed");
                }

                // TODO underflow/overflow
                let change = (sel.e - sel.b) as isize;
                modifier -= change;

                if c.anchor_right() {
                    c.shift_by(change);
                }
            }

            c.clear_both();
            diff_len += cursor_specific_diff_len;
            debug_assert!(c.check_invariant());
        }
    };
    cs.reduce_right();
    debug_assert!(cs.check_invariants());
    (diff_len, res)
}

// Returns tuple:
//      first is number of chars that changed (inserted, removed, changed), or 0 in case of UNDO/REDO
//      FALSE iff the command results in no-op.
pub fn apply_cem(cem: CommonEditMsg,
                 cs: &mut CursorSet,
                 rope: &mut dyn Buffer,
                 page_height: usize,
                 clipboard: Option<&ClipboardRef>) -> (usize, bool) {
    let res = match cem {
        CommonEditMsg::Char(char) => {
            // TODO optimise
            insert_to_rope(cs, rope, None, char.to_string().as_str())
        }
        CommonEditMsg::CursorUp { selecting } => {
            (0, cs.move_vertically_by(rope, -1, selecting))
        }
        CommonEditMsg::CursorDown { selecting } => {
            (0, cs.move_vertically_by(rope, 1, selecting))
        }
        CommonEditMsg::CursorLeft { selecting } => {
            (0, cs.move_left(selecting))
        }
        CommonEditMsg::CursorRight { selecting } => {
            (0, cs.move_right(rope, selecting))
        }
        CommonEditMsg::Backspace => {
            let mut res = false;
            let mut modifier: isize = 0;
            let mut diff_len: usize = 0;
            for c in cs.iter_mut() {
                c.shift_by(modifier);

                if cfg!(debug_assertions) {
                    // equal is OK.
                    if c.a > rope.len_chars() {
                        error!("cursor beyond length of rope: {} > {}", c.a, rope.len_chars());
                        continue;
                    }
                }

                if let Some(sel) = c.s {
                    if rope.remove(sel.b, sel.e) {
                        res |= true;
                        diff_len += sel.len();
                    } else {
                        warn!("expected to remove non-empty item substring but failed");
                    }

                    // TODO underflow/overflow
                    let change = (sel.e - sel.b) as isize;
                    modifier -= change;

                    if c.anchor_right() {
                        c.shift_by(change);
                    }
                } else {
                    if c.a == 0 {
                        continue;
                    }

                    if rope.remove(c.a - 1, c.a) {
                        res |= true;
                        diff_len += 1;
                    } else {
                        warn!("expected to remove char but failed");
                    }
                    modifier -= 1;
                    c.shift_by(-1);
                }
                c.clear_both();
                debug_assert!(c.check_invariant());
            };

            cs.reduce_left();
            debug_assert!(cs.check_invariants());
            (diff_len, res)
        }
        CommonEditMsg::LineBegin { selecting } => {
            (0, cs.move_home(rope, selecting))
        }
        CommonEditMsg::LineEnd { selecting } => {
            (0, cs.move_end(rope, selecting))
        }
        CommonEditMsg::WordBegin { selecting } => {
            (0, cs.word_begin_default(rope, selecting))
        }
        CommonEditMsg::WordEnd { selecting } => {
            (0, cs.word_end_default(rope, selecting))
        }
        CommonEditMsg::PageUp { selecting } => {
            if page_height > PAGE_HEIGHT_LIMIT {
                error!("received PageUp of page_height {}, ignoring.", page_height);
                (0, false)
            } else {
                (0, cs.move_vertically_by(rope, -(page_height as isize), selecting))
            }
        }
        CommonEditMsg::PageDown { selecting } => {
            if page_height > PAGE_HEIGHT_LIMIT {
                error!("received PageDown of page_height {}, ignoring.", page_height);
                (0, false)
            } else {
                (0, cs.move_vertically_by(rope, page_height as isize, selecting))
            }
        }
        CommonEditMsg::Delete => {
            let mut res = false;
            let mut modifier: isize = 0;
            let mut diff_len: usize = 0;
            for c in cs.iter_mut() {
                c.shift_by(modifier);

                if cfg!(debug_assertions) {
                    // equal is OK.
                    if c.a > rope.len_chars() {
                        error!("cursor beyond length of rope: {} > {}", c.a, rope.len_chars());
                        continue;
                    }
                }

                if let Some(sel) = c.s {
                    if rope.remove(sel.b, sel.e) {
                        res |= true;
                        diff_len += sel.len();
                    } else {
                        warn!("expected to remove non-empty item substring but failed");
                    }

                    let change = (sel.e - sel.b) as isize;
                    modifier -= change;

                    if c.anchor_right() {
                        c.shift_by(change);
                    }
                } else {
                    if c.a == rope.len_chars() {
                        continue;
                    }

                    if rope.remove(c.a, c.a + 1) {
                        res |= true;
                        diff_len += 1;
                    } else {
                        warn!("expected to remove char but failed");
                    }
                    modifier -= 1;
                }
                c.clear_both();
                debug_assert!(c.check_invariant());
            };

            cs.reduce_left();
            debug_assert!(cs.check_invariants());
            (diff_len, res)
        }
        CommonEditMsg::Copy => {
            if let Some(clipboard) = clipboard {
                let mut contents = String::new();
                for c in cs.iter() {
                    if !contents.is_empty() {
                        contents.push('\n');
                    }

                    if let Some(sel) = c.s {
                        for c in rope.chars().skip(sel.b).take(sel.e - sel.b) {
                            contents.push(c);
                        }
                    }
                }

                (0, clipboard.set(contents))
            } else {
                warn!("copy without a clipboard, ignoring");
                (0, false)
            }
        }
        CommonEditMsg::Paste => {
            if let Some(clipboard) = clipboard {
                let cursor_count = cs.set().len();
                let contents = clipboard.get();
                if contents.is_empty() {
                    warn!("not pasting empty contents");
                    (0, false)
                } else {
                    let split_lines = contents.lines().count() == cursor_count;
                    // easy, each cursor gets full copy
                    if !split_lines {
                        insert_to_rope(cs, rope, None, &contents)
                    } else {
                        let mut res = false;
                        let mut it = contents.lines();
                        let mut common_idx: usize = 0;
                        let mut diff_len: usize = 0;
                        while let Some(line) = it.next() {
                            let (dl, r) = insert_to_rope(cs, rope, Some(common_idx), line);
                            res |= r;
                            diff_len += dl;
                            common_idx += 1;
                        }

                        (diff_len, res)
                    }
                }
            } else {
                warn!("paste without a clipboard, ignoring");
                (0, false)
            }
        }
        CommonEditMsg::Undo => {
            (0, rope.undo())
        }
        CommonEditMsg::Redo => {
            (0, rope.redo())
        }
    };

    debug_assert!(cs.check_invariants());

    res
}

// This maps a cme into a direction that the cursor has (probably) moved. It's used to update
// scrolling information.
pub fn cme_to_direction(cme: CommonEditMsg) -> Option<Arrow> {
    match cme {
        CommonEditMsg::Char(_) => Some(Arrow::Right),
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
    }
}