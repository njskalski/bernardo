use log::{debug, error};
use ropey::Rope;

use crate::io::keys::Key;
use crate::Keycode;
use crate::primitives::cursor_set::{Cursor, CursorSet};
use crate::text::buffer::Buffer;

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
        Key { keycode, modifiers } => {
            match keycode {
                Keycode::Char(c) => Some(CommonEditMsg::Char(c)),
                Keycode::ArrowUp => Some(CommonEditMsg::CursorUp { selecting: key.modifiers.SHIFT }),
                Keycode::ArrowDown => Some(CommonEditMsg::CursorDown { selecting: key.modifiers.SHIFT }),
                Keycode::ArrowLeft => Some(CommonEditMsg::CursorLeft { selecting: key.modifiers.SHIFT }),
                Keycode::ArrowRight => Some(CommonEditMsg::CursorRight { selecting: key.modifiers.SHIFT }),
                Keycode::Enter => {
                    debug!("mapping Keycode:Space to Char('\\n')");
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
                    debug!("mapping Keycode:Space to Char('\\t')");
                    Some(CommonEditMsg::Char('\t'))
                }
                Keycode::Delete => Some(CommonEditMsg::Delete),
                _ => None
            }
        }
    }
}

// Returns FALSE if the command results in no-op.
// TODO the result is mocked, not implemented properly
pub fn apply_cme(cem: CommonEditMsg, cs: &mut CursorSet, rope: &mut dyn Buffer) -> bool {
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

            cs.move_right_by(rope, 1, false);
            true //TODO single cursor should return false on impossible
        }
        CommonEditMsg::CursorUp { selecting } => {
            cs.move_vertically_by(rope, -1);
            true//TODO single cursor should return false on impossible
        }
        CommonEditMsg::CursorDown { selecting } => {
            cs.move_vertically_by(rope, 1);
            true//TODO single cursor should return false on impossible
        }
        CommonEditMsg::CursorLeft { selecting } => {
            cs.move_left(selecting);
            true//TODO single cursor should return false on impossible
        }
        CommonEditMsg::CursorRight { selecting } => {
            cs.move_right(rope, selecting);
            true
        }
        CommonEditMsg::Backspace => {
            cs.backspace(rope)
        }
        CommonEditMsg::LineBegin { selecting } => {
            cs.home(rope)
        }
        CommonEditMsg::LineEnd { selecting } => {
            cs.end(rope)
        }
        // CommonEditMsg::WordBegin => {}
        // CommonEditMsg::WordEnd => {}
        // CommonEditMsg::PageUp => {}
        // CommonEditMsg::PageDown => {}
        // CommonEditMsg::Delete => {}
        e => {
            debug!("unhandled common edit msg {:?}", e);
            false
        }
    }
}