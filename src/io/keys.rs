use crate::io::keys::Key::ArrowLeft;
use log::debug;
use termion::event::Key as TKey;

#[derive(Debug, Clone, Copy)]
pub enum Key {
    Letter(char),
    CtrlLetter(char),
    AltLetter(char),
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

impl From<TKey> for Key {
    fn from(tk: TKey) -> Self {
        match tk {
            TKey::Backspace => Key::Backspace,
            TKey::Left => Key::ArrowLeft,
            TKey::Right => Key::ArrowRight,
            TKey::Up => Key::ArrowUp,
            TKey::Down => Key::ArrowDown,
            TKey::Home => Key::Home,
            TKey::End => Key::End,
            TKey::PageUp => Key::PageUp,
            TKey::PageDown => Key::PageDown,
            TKey::BackTab => Key::Tab,
            TKey::Delete => Key::Delete,
            TKey::Insert => Key::Insert,
            TKey::F(u) => Key::F(u),
            TKey::Char(c) => Key::Letter(c),
            TKey::Alt(c) => Key::AltLetter(c),
            TKey::Ctrl(c) => Key::CtrlLetter(c),
            TKey::Null => Key::Null,
            TKey::Esc => Key::Esc,
            _ => {
                debug!(
                    "Faild Termion event conversion nsupported symbol [{:?}]",
                    tk
                );
                Key::Unhandled
            }
        }
    }
}
