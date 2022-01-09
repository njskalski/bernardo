use log::{debug, error};
use ropey::Rope;

use crate::io::keys::Key;
use crate::Keycode;
use crate::primitives::cursor_set::{Cursor, CursorSet};
use crate::text::buffer::Buffer;

#[derive(Debug, Clone, Copy)]
pub enum CommonEditMsg {
    Char(char),
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    Backspace,
    LineBegin,
    LineEnd,
    WordBegin,
    WordEnd,
    PageUp,
    PageDown,
    Delete,
}

// This is where the mapping of keys to Msgs is
pub fn key_to_edit_msg(key: Key) -> Option<CommonEditMsg> {
    match key {
        Key { keycode, modifiers } => {
            match keycode {
                Keycode::Char(c) => Some(CommonEditMsg::Char(c)),
                Keycode::ArrowUp => Some(CommonEditMsg::CursorUp),
                Keycode::ArrowDown => Some(CommonEditMsg::CursorDown),
                Keycode::ArrowLeft => Some(CommonEditMsg::CursorLeft),
                Keycode::ArrowRight => Some(CommonEditMsg::CursorRight),
                Keycode::Enter => {
                    debug!("mapping Keycode:Space to Char('\\n')");
                    Some(CommonEditMsg::Char('\n'))
                }
                Keycode::Space => {
                    debug!("mapping Keycode:Space to Char(' ')");
                    Some(CommonEditMsg::Char(' '))
                }
                Keycode::Backspace => Some(CommonEditMsg::Backspace),
                Keycode::Home => Some(CommonEditMsg::LineBegin),
                Keycode::End => Some(CommonEditMsg::LineEnd),
                Keycode::PageUp => Some(CommonEditMsg::PageUp),
                Keycode::PageDown => Some(CommonEditMsg::PageDown),
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

            cs.move_right_by(rope, 1);
            true //TODO single cursor should return false on impossible
        }
        CommonEditMsg::CursorUp => {
            cs.move_vertically_by(rope, -1);
            true//TODO single cursor should return false on impossible
        }
        CommonEditMsg::CursorDown => {
            cs.move_vertically_by(rope, 1);
            true//TODO single cursor should return false on impossible
        }
        CommonEditMsg::CursorLeft => {
            cs.move_left();
            true//TODO single cursor should return false on impossible
        }
        CommonEditMsg::CursorRight => {
            cs.move_right(rope);
            true
        }
        CommonEditMsg::Backspace => {
            for c in cs.iter().rev() {
                rope.remove(c.a, c.a + 1);
            }
            true
        }
        CommonEditMsg::LineBegin => {
            cs.home(rope)
        }
        CommonEditMsg::LineEnd => {
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