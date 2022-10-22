use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::KeyEvent as CKey;
use log::error;

use crate::io::keys::{Key, Keycode, Modifiers};

impl From<CKey> for Key {
    fn from(ckey: CKey) -> Self {
        return match ckey {
            KeyEvent { code, modifiers, kind, state } => {
                let kc: Keycode = match code {
                    KeyCode::Backspace => Keycode::Backspace,
                    KeyCode::Enter => Keycode::Enter,
                    KeyCode::Left => Keycode::ArrowLeft,
                    KeyCode::Right => Keycode::ArrowRight,
                    KeyCode::Up => Keycode::ArrowUp,
                    KeyCode::Down => Keycode::ArrowDown,
                    KeyCode::Home => Keycode::Home,
                    KeyCode::End => Keycode::End,
                    KeyCode::PageUp => Keycode::PageUp,
                    KeyCode::PageDown => Keycode::PageDown,
                    KeyCode::Tab => Keycode::Tab,
                    KeyCode::BackTab => Keycode::Tab,
                    KeyCode::Delete => Keycode::Delete,
                    KeyCode::Insert => Keycode::Insert,
                    KeyCode::F(f) => Keycode::F(f),
                    KeyCode::Char(' ') => Keycode::Space,
                    KeyCode::Char(char) => Keycode::Char(char),
                    KeyCode::Null => Keycode::Null,
                    KeyCode::Esc => Keycode::Esc,
                    keycode => {
                        error!("unhandled keycode {:?}", keycode);
                        Keycode::Unhandled
                    }
                };

                let md: Modifiers = Modifiers::new(
                    modifiers.contains(KeyModifiers::ALT),
                    modifiers.contains(KeyModifiers::CONTROL),
                    modifiers.contains(KeyModifiers::SHIFT),
                );

                Key {
                    keycode: kc,
                    modifiers: md,
                }
            }
        };
    }
}
