use log::{debug, error};

use crate::io::keys::Key;
use crate::Keycode;
use crate::primitives::arrow::Arrow;
use crate::primitives::cursor_set::CursorSet;
use crate::text::buffer::Buffer;

// this is a completely arbitrary number against which I compare Page length, to avoid under/overflow while casting to isize safely.
const PAGE_HEIGHT_LIMIT: usize = 2000;

#[derive(Debug, Clone, Copy)]
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
}

// This is where the mapping of keys to Msgs is
pub fn key_to_edit_msg(key: Key) -> Option<CommonEditMsg> {
    match key {
        Key { keycode, modifiers: _ } => {
            match keycode {
                Keycode::Char(c) => Some(CommonEditMsg::Char(c)),
                Keycode::ArrowUp => Some(CommonEditMsg::CursorUp { selecting: key.modifiers.SHIFT }),
                Keycode::ArrowDown => Some(CommonEditMsg::CursorDown { selecting: key.modifiers.SHIFT }),
                Keycode::ArrowLeft => {
                    if key.modifiers.CTRL {
                        Some(CommonEditMsg::WordBegin { selecting: key.modifiers.SHIFT })
                    } else {
                        Some(CommonEditMsg::CursorLeft { selecting: key.modifiers.SHIFT })
                    }
                },
                Keycode::ArrowRight => {
                    if key.modifiers.CTRL {
                        Some(CommonEditMsg::WordEnd { selecting: key.modifiers.SHIFT })
                    } else {
                        Some(CommonEditMsg::CursorRight { selecting: key.modifiers.SHIFT })
                    }
                },
                Keycode::Enter => {
                    debug!("mapping Keycode:Enter to Char('\\n')");
                    Some(CommonEditMsg::Char('\n'))
                }
                Keycode::Space => {
                    debug!("mapping Keycode:Space to Char(' ')");
                    Some(CommonEditMsg::Char(' '))
                }
                Keycode::Backspace => Some(CommonEditMsg::Backspace),
                Keycode::Home => Some(CommonEditMsg::LineBegin { selecting: key.modifiers.SHIFT }),
                Keycode::End => Some(CommonEditMsg::LineEnd { selecting: key.modifiers.SHIFT }),
                Keycode::PageUp => Some(CommonEditMsg::PageUp { selecting: key.modifiers.SHIFT }),
                Keycode::PageDown => Some(CommonEditMsg::PageDown { selecting: key.modifiers.SHIFT }),
                Keycode::Tab => {
                    debug!("mapping Keycode:Tab to Char('\\t')");
                    Some(CommonEditMsg::Char('\t'))
                }
                Keycode::Delete => Some(CommonEditMsg::Delete),
                _ => None
            }
        }
    }
}

// Returns FALSE if the command results in no-op.
pub fn apply_cme(cem: CommonEditMsg, cs: &mut CursorSet, rope: &mut dyn Buffer, page_height: usize) -> bool {
    match cem {
        CommonEditMsg::Char(char) => {
            for c in cs.iter() {
                if cfg!(debug_assertions) {
                    if c.a > rope.len_chars() {
                        error!("cursor beyond length of rope: {} > {}", c.a, rope.len_chars());
                    }
                }

                rope.insert_char(c.a, char);
            };

            cs.move_right_by(rope, 1, false)
        }
        CommonEditMsg::CursorUp { selecting } => {
            cs.move_vertically_by(rope, -1, selecting)
        }
        CommonEditMsg::CursorDown { selecting } => {
            cs.move_vertically_by(rope, 1, selecting)
        }
        CommonEditMsg::CursorLeft { selecting } => {
            cs.move_left(selecting)
        }
        CommonEditMsg::CursorRight { selecting } => {
            cs.move_right(rope, selecting)
        }
        CommonEditMsg::Backspace => {
            cs.backspace(rope)
        }
        CommonEditMsg::LineBegin { selecting } => {
            cs.home(rope, selecting)
        }
        CommonEditMsg::LineEnd { selecting } => {
            cs.end(rope, selecting)
        }
        CommonEditMsg::WordBegin { selecting } => {
            cs.word_begin_default(rope, selecting)
        }
        CommonEditMsg::WordEnd { selecting } => {
            cs.word_end_default(rope, selecting)
        }
        CommonEditMsg::PageUp { selecting } => {
            if page_height > PAGE_HEIGHT_LIMIT {
                error!("received PageUp of page_height {}, ignoring.", page_height);
                false
            } else {
                cs.move_vertically_by(rope, -(page_height as isize), selecting)
            }
        }
        CommonEditMsg::PageDown { selecting } => {
            if page_height > PAGE_HEIGHT_LIMIT {
                error!("received PageDown of page_height {}, ignoring.", page_height);
                false
            } else {
                cs.move_vertically_by(rope, page_height as isize, selecting)
            }
        }
        CommonEditMsg::Delete => {
            let mut res = false;
            for c in cs.iter() {
                if cfg!(debug_assertions) {
                    if c.a > rope.len_chars() {
                        error!("cursor beyond length of rope: {} > {}", c.a, rope.len_chars());
                    }
                }

                res |= rope.remove(c.a, c.a + 1);
            };

            res
        }
        e => {
            debug!("unhandled common edit msg {:?}", e);
            false
        }
    }
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
    }
}