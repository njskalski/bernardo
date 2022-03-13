use std::fmt::{Display, Formatter};

use crate::experiments::focus_group::FocusUpdate;

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

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub struct Key {
    pub keycode: Keycode,
    pub modifiers: Modifiers,
}

impl Key {
    pub fn no_modifiers(&self) -> bool {
        !(self.modifiers.ALT || self.modifiers.CTRL || self.modifiers.SHIFT)
    }

    pub fn as_focus_update(&self) -> Option<FocusUpdate> {
        if self.modifiers.ALT == false {
            return None;
        }

        return match self.keycode {
            Keycode::ArrowUp => Some(FocusUpdate::Up),
            Keycode::ArrowDown => Some(FocusUpdate::Down),
            Keycode::ArrowLeft => Some(FocusUpdate::Left),
            Keycode::ArrowRight => Some(FocusUpdate::Right),
            _ => None
        };
    }
}

impl Keycode {
    pub fn is_arrow(&self) -> bool {
        return *self == Keycode::ArrowRight ||
            *self == Keycode::ArrowLeft ||
            *self == Keycode::ArrowUp ||
            *self == Keycode::ArrowDown;
    }
}

impl Display for Keycode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

