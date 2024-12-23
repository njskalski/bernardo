use std::fmt::Formatter;
use std::str::FromStr;

use log::error;
use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::experiments::focus_group::FocusUpdate;
use crate::io::input_event::InputEvent;

// TODO (hardening) here potentially impossible combinations, like ALT+LeftAlt are deserializable,
// should be fixed someday
// TODO (hardening) also, for some reason console does not support
// combinations like shift+ctrl+s, I need to warn users to not try that
const ALT_PLUS: &str = "ALT+";
const CTRL_PLUS: &str = "CTRL+";
const SHIFT_PLUS: &str = "SHIFT+";
const ALT: &str = "ALT";
const CTRL: &str = "CTRL";
const SHIFT: &str = "SHIFT";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Modifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
}

impl Default for Modifiers {
    fn default() -> Self {
        Modifiers {
            alt: false,
            ctrl: false,
            shift: false,
        }
    }
}

impl Modifiers {
    pub fn new(alt: bool, ctrl: bool, shift: bool) -> Modifiers {
        Modifiers { alt, ctrl, shift }
    }

    pub fn is_empty(&self) -> bool {
        !(self.alt || self.ctrl || self.shift)
    }

    pub fn just_alt(&self) -> bool {
        self.alt && !self.ctrl && !self.shift
    }

    pub fn just_ctrl(&self) -> bool {
        !self.alt && self.ctrl && !self.shift
    }

    pub fn just_shift(&self) -> bool {
        !self.alt && !self.ctrl && self.shift
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Key {
    pub keycode: Keycode,
    pub modifiers: Modifiers,
}

impl ToString for Key {
    fn to_string(&self) -> String {
        let s = match ron::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                error!("failed serializing key to string {:?} because {}", self, e);
                return "".to_string();
            }
        };

        if s.starts_with("\"") && s.ends_with("\"") {
            s[1..s.len() - 1].to_string()
        } else {
            s
        }
    }
}

impl Key {
    pub fn no_modifiers(&self) -> bool {
        !(self.modifiers.alt || self.modifiers.ctrl || self.modifiers.shift)
    }

    pub fn as_focus_update(&self) -> Option<FocusUpdate> {
        return match self.keycode {
            Keycode::ArrowUp => Some(FocusUpdate::Up),
            Keycode::ArrowDown => Some(FocusUpdate::Down),
            Keycode::ArrowLeft => Some(FocusUpdate::Left),
            Keycode::ArrowRight => Some(FocusUpdate::Right),
            _ => None,
        };
    }

    pub fn with_alt(self) -> Self {
        Key {
            modifiers: Modifiers {
                alt: true,
                ctrl: self.modifiers.ctrl,
                shift: self.modifiers.shift,
            },
            ..self
        }
    }

    pub fn with_ctrl(self) -> Self {
        Key {
            modifiers: Modifiers {
                alt: self.modifiers.alt,
                ctrl: true,
                shift: self.modifiers.shift,
            },
            ..self
        }
    }

    pub fn with_shift(self) -> Self {
        Key {
            modifiers: Modifiers {
                alt: self.modifiers.alt,
                ctrl: self.modifiers.ctrl,
                shift: true,
            },
            ..self
        }
    }

    pub fn to_input_event(self) -> InputEvent {
        InputEvent::KeyInput(self)
    }
}

impl Keycode {
    pub fn is_arrow(&self) -> bool {
        return *self == Keycode::ArrowRight || *self == Keycode::ArrowLeft || *self == Keycode::ArrowUp || *self == Keycode::ArrowDown;
    }

    pub fn to_key(self) -> Key {
        Key {
            keycode: self,
            modifiers: Modifiers::default(),
        }
    }

    // True for all characters, but not arrows, F1-12 etc.
    pub fn is_symbol(&self) -> bool {
        match self {
            Keycode::Char(_) => true,
            _ => false,
        }
    }
}

impl ToString for Keycode {
    fn to_string(&self) -> String {
        match self {
            Keycode::Char(c) => c.to_lowercase().to_string(),
            Keycode::F(x) => format!("F{}", x),
            _ => format!("{:?}", self),
        }
    }
}

impl FromStr for Keycode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ArrowUp" => Ok(Keycode::ArrowUp),
            "ArrowDown" => Ok(Keycode::ArrowDown),
            "ArrowLeft" => Ok(Keycode::ArrowLeft),
            "ArrowRight" => Ok(Keycode::ArrowRight),
            "Enter" => Ok(Keycode::Enter),
            "Space" => Ok(Keycode::Space),
            "LeftAlt" => Ok(Keycode::LeftAlt),
            "RightAlt" => Ok(Keycode::RightAlt),
            "LeftCtrl" => Ok(Keycode::LeftCtrl),
            "RightCtrl" => Ok(Keycode::RightCtrl),
            "Backspace" => Ok(Keycode::Backspace),
            "Home" => Ok(Keycode::Home),
            "End" => Ok(Keycode::End),
            "PageUp" => Ok(Keycode::PageUp),
            "PageDown" => Ok(Keycode::PageDown),
            "Tab" => Ok(Keycode::Tab),
            "Delete" => Ok(Keycode::Delete),
            "Insert" => Ok(Keycode::Insert),
            // "Null" => Ok(Keycode::
            "Esc" => Ok(Keycode::Esc),
            other => {
                if (other.starts_with("F") || other.starts_with("f")) && other.len() > 1 {
                    match u8::from_str(&other[1..]) {
                        Ok(i) => {
                            if i < 16 {
                                Ok(Keycode::F(i))
                            } else {
                                Err(())
                            }
                        }
                        Err(_) => Err(()),
                    }
                } else if other.len() == 1 {
                    let x = other.chars().next().unwrap().to_lowercase().next().unwrap();
                    Ok(Keycode::Char(x))
                } else {
                    Err(())
                }
            }
        }
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let alt = if self.modifiers.alt { ALT_PLUS } else { "" };
        let ctrl = if self.modifiers.ctrl { CTRL_PLUS } else { "" };
        let shift = if self.modifiers.shift { SHIFT_PLUS } else { "" };

        serializer.serialize_str(&format!(
            "{}{}{}{}",
            alt,
            ctrl,
            shift,
            // keycode_unescaped,
            self.keycode.to_string(),
        ))
    }
}

struct KeyVisitor;

impl<'de> Visitor<'de> for KeyVisitor {
    type Value = Key;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a key description in \"{ALT+}?{CTRL+}?{SHIFT+}?KeyCode)\" format")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        //TODO (cleanup) this is over tolerant, we allow multiple shifts, ctrls, different order, whitespaces etc

        let cleaned: String = v.chars().filter(|c| !c.is_whitespace()).collect();

        let mut mods = Modifiers {
            alt: false,
            ctrl: false,
            shift: false,
        };

        let mut keycode: Option<Keycode> = None;

        for part in cleaned.split("+") {
            match part {
                ALT => mods.alt = true,
                CTRL => mods.ctrl = true,
                SHIFT => mods.shift = true,
                keycode_str => {
                    // TODO I have no clue how to write it better and I refuse to learn tonight.
                    match Keycode::from_str(keycode_str) {
                        Ok(k) => keycode = Some(k),
                        Err(_) => return Err(de::Error::missing_field("keycode")),
                    }
                }
            }
        }

        match keycode {
            Some(keycode) => Ok(Key { keycode, modifiers: mods }),
            None => Err(serde::de::Error::missing_field("keycode")),
        }
    }
}

impl<'a> Deserialize<'a> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_str(KeyVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_to_string() {
        assert_eq!(Keycode::Enter.to_string(), "Enter".to_string());
        assert_eq!(Keycode::F(1).to_string(), "F1".to_string());
        assert_eq!(Keycode::Char('a').to_string(), "a".to_string());
        assert_eq!(Keycode::Char('X').to_string(), "x".to_string());
        assert_eq!(Keycode::Char('\'').to_string(), "'".to_string());
    }

    #[test]
    fn test_keycode_from_str() {
        assert_eq!(Keycode::from_str("Enter"), Ok(Keycode::Enter));
        assert_eq!(Keycode::from_str("F11"), Ok(Keycode::F(11)));
        assert_eq!(Keycode::from_str("-"), Ok(Keycode::Char('-')));
        assert_eq!(Keycode::from_str("'"), Ok(Keycode::Char('\'')));
        assert_eq!(Keycode::from_str("nothing"), Err(()));
    }

    #[test]
    fn test_key_serialize() {
        assert_eq!(
            ron::to_string(&Key {
                keycode: Keycode::ArrowUp,
                modifiers: Modifiers {
                    alt: false,
                    ctrl: true,
                    shift: false,
                },
            }),
            Ok((r#""CTRL+ArrowUp""#).to_string())
        );
        assert_eq!(
            ron::to_string(&Key {
                keycode: Keycode::Delete,
                modifiers: Modifiers {
                    alt: true,
                    ctrl: true,
                    shift: false,
                },
            }),
            Ok(r#""ALT+CTRL+Delete""#.to_string())
        );
        assert_eq!(
            ron::to_string(&Key {
                keycode: Keycode::Char('x'),
                modifiers: Modifiers {
                    alt: false,
                    ctrl: false,
                    shift: false,
                },
            }),
            Ok(r#""x""#.to_string())
        );
        assert_eq!(
            ron::to_string(&Key {
                keycode: Keycode::Char('x'),
                modifiers: Modifiers {
                    alt: false,
                    ctrl: true,
                    shift: false,
                },
            }),
            Ok(r#""CTRL+x""#.to_string())
        );
    }

    #[test]
    fn test_key_deserialize() {
        assert_eq!(
            ron::from_str("\"CTRL+ArrowUp\""),
            Ok(Key {
                keycode: Keycode::ArrowUp,
                modifiers: Modifiers {
                    alt: false,
                    ctrl: true,
                    shift: false,
                },
            })
        );
        assert_eq!(
            ron::from_str("\"CTRL+q\""),
            Ok(Key {
                keycode: Keycode::Char('q'),
                modifiers: Modifiers {
                    alt: false,
                    ctrl: true,
                    shift: false,
                },
            })
        );
        assert_eq!(
            ron::from_str("\"ALT+CTRL+Delete\""),
            Ok(Key {
                keycode: Keycode::Delete,
                modifiers: Modifiers {
                    alt: true,
                    ctrl: true,
                    shift: false,
                },
            })
        );
    }

    #[test]
    fn test_key_ser_de() {
        let keys = vec![
            Keycode::Char('s').to_key().with_ctrl(),
            Keycode::Char('d').to_key().with_ctrl(),
            Keycode::Char('e').to_key().with_ctrl(),
            Keycode::Char('f').to_key().with_ctrl(),
            Keycode::Char('r').to_key().with_ctrl(),
            Keycode::Esc.to_key(),
        ];

        for key in keys {
            let r = ron::to_string(&key);
            assert_eq!(r.as_ref().err(), None);
            let s = r.unwrap();

            let kr = ron::from_str::<Key>(&s);
            assert_eq!(kr.as_ref().err(), None, "{}", s);
            let k = kr.unwrap();

            assert_eq!(key, k);
        }
    }
}
