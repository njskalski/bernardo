use log::{debug, warn};
use termion::event::Key as TKey;

use crate::io::keys::{Key, Keycode, Modifiers};

impl From<TKey> for Key {
    fn from(tk: TKey) -> Self {
        let mut kc: Keycode = match tk {
            TKey::Backspace => Keycode::Backspace,
            TKey::Left => Keycode::ArrowLeft,
            TKey::Right => Keycode::ArrowRight,
            TKey::Up => Keycode::ArrowUp,
            TKey::Down => Keycode::ArrowDown,
            TKey::Home => Keycode::Home,
            TKey::End => Keycode::End,
            TKey::PageUp => Keycode::PageUp,
            TKey::PageDown => Keycode::PageDown,
            TKey::BackTab => Keycode::Tab,
            TKey::Delete => Keycode::Delete,
            TKey::Insert => Keycode::Insert,
            TKey::F(u) => Keycode::F(u),
            TKey::Char(c) => Keycode::Char(c),
            TKey::Null => Keycode::Null,
            TKey::Esc => Keycode::Esc,
            _ => {
                debug!("Failed Termion event conversion unsupported symbol [{:?}]",tk);
                Keycode::Unhandled
            }
        };

        let mut md: Modifiers = match tk {
            TKey::Alt(_) => Modifiers::new(true, false, false),
            TKey::Ctrl(_) => Modifiers::new(false, true, false),
            _ => Modifiers::new(false, false, false),
        };

        if let Keycode::Char(c) = kc {
            if c.is_uppercase() {
                md.SHIFT = true;
                let lowercase_str = c.to_lowercase().to_string();
                if lowercase_str.len() != 1 {
                    warn!("Unsupported lowercase mapping {}", lowercase_str);
                    kc = Keycode::Unhandled;
                } else {
                    let c = lowercase_str.chars().nth(0).unwrap();
                    kc = Keycode::Char(c);
                }
            }
        }

        Key {
            keycode: kc,
            modifiers: md,
        }
    }
}
