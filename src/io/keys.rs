use std::fmt::{Display, Formatter};

use crossterm::event::{KeyEvent as CKey, KeyEvent};
use log::debug;

use crate::experiments::focus_group::FocusUpdate;
use crate::io::keys::Keycode::ArrowLeft;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keycode {
    Char(char),
    F(u8),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Enter,
    Space,
    LeftAlt,
    RightAlt,
    LeftCtrl,
    RightCtrl,
    Backspace,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Delete,
    Insert,
    Null,
    Esc,
    Unhandled,
}

pub struct Modifiers {
    pub ALT: bool,
    pub CTRL: bool,
    pub SHIFT: bool,
}

impl Modifiers {
    pub fn new(alt: bool, ctrl: bool, shift: bool) -> Modifiers {
        Modifiers {
            ALT: alt,
            CTRL: ctrl,
            SHIFT: shift,
        }
    }
}

pub struct Key {
    pub keycode: Keycode,
    pub modifiers: Modifiers,
}

impl Keycode {
    pub fn is_arrow(&self) -> bool {
        return *self == Keycode::ArrowRight ||
            *self == Keycode::ArrowLeft ||
            *self == Keycode::ArrowUp ||
            *self == Keycode::ArrowDown
    }

    pub fn as_focus_update(&self) -> Option<FocusUpdate> {
        return match self {
            Keycode::ArrowUp => Some(FocusUpdate::Up),
            Keycode::ArrowDown => Some(FocusUpdate::Down),
            Keycode::ArrowLeft => Some(FocusUpdate::Left),
            Keycode::ArrowRight => Some(FocusUpdate::Right),
            _ => None
        }
    }
}

impl Display for Keycode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

