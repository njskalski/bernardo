use crate::io::keys::{Key, Keycode};
use crate::primitives::arrow::Arrow;

#[derive(Clone, Copy, Debug)]
pub enum ScrollEnum {
    Arrow(Arrow),
    Home,
    End,
    PageUp,
    PageDown,
}

impl ScrollEnum {
    pub fn from_key(key: Key) -> Option<ScrollEnum> {
        if !key.no_modifiers() {
            return None;
        }

        #[allow(unreachable_patterns)]
        match key {
            Key { keycode, modifiers: _ } => {
                match keycode {
                    Keycode::ArrowUp => Some(ScrollEnum::Arrow(Arrow::Up)),
                    Keycode::ArrowDown => Some(ScrollEnum::Arrow(Arrow::Down)),
                    Keycode::ArrowLeft => Some(ScrollEnum::Arrow(Arrow::Left)),
                    Keycode::ArrowRight => Some(ScrollEnum::Arrow(Arrow::Right)),
                    Keycode::Home => Some(ScrollEnum::Home),
                    Keycode::End => Some(ScrollEnum::End),
                    Keycode::PageUp => Some(ScrollEnum::PageUp),
                    Keycode::PageDown => Some(ScrollEnum::PageDown),
                    _ => None
                }
            }
            _ => {
                None
            }
        }
    }
}