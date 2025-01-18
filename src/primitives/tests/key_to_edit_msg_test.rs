use crate::io::keys::Keycode;
use crate::primitives::common_edit_msgs::{key_to_edit_msg, CommonEditMsg};
use crate::primitives::tests::test_helpers::{generate_pseudo_random_edit_msgs_config, generate_random_key};
use crossterm::event::KeyCode;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[test]
fn test_random_keybindings() {
    let mut rng = StdRng::seed_from_u64(12345);
    let keybindings = generate_pseudo_random_edit_msgs_config();

    for _ in 0..100 {
        let key = generate_random_key(&mut rng);
        if key == keybindings.copy {
            assert_eq!(key_to_edit_msg(key, &keybindings), Some(CommonEditMsg::Copy));
        } else if key == keybindings.paste {
            assert_eq!(key_to_edit_msg(key, &keybindings), Some(CommonEditMsg::Paste));
        } else if key == keybindings.undo {
            assert_eq!(key_to_edit_msg(key, &keybindings), Some(CommonEditMsg::Undo));
        } else if key == keybindings.redo {
            assert_eq!(key_to_edit_msg(key, &keybindings), Some(CommonEditMsg::Redo));
        } else if key == keybindings.cursor_up {
            assert_eq!(
                key_to_edit_msg(key, &keybindings),
                Some(CommonEditMsg::CursorUp {
                    selecting: key.modifiers.shift,
                })
            );
        } else if key == keybindings.cursor_down {
            assert_eq!(
                key_to_edit_msg(key, &keybindings),
                Some(CommonEditMsg::CursorDown {
                    selecting: key.modifiers.shift,
                })
            );
        } else if key == keybindings.cursor_left {
            assert_eq!(
                key_to_edit_msg(key, &keybindings),
                Some(CommonEditMsg::CursorLeft {
                    selecting: key.modifiers.shift,
                })
            );
        } else if key == keybindings.cursor_right {
            assert_eq!(
                key_to_edit_msg(key, &keybindings),
                Some(CommonEditMsg::CursorRight {
                    selecting: key.modifiers.shift,
                })
            );
        } else if key == keybindings.word_begin {
            assert_eq!(
                key_to_edit_msg(key, &keybindings),
                Some(CommonEditMsg::WordBegin {
                    selecting: key.modifiers.shift,
                })
            );
        } else if key == keybindings.word_end {
            assert_eq!(
                key_to_edit_msg(key, &keybindings),
                Some(CommonEditMsg::WordEnd {
                    selecting: key.modifiers.shift,
                })
            );
        } else if key == keybindings.backspace {
            assert_eq!(key_to_edit_msg(key, &keybindings), Some(CommonEditMsg::Backspace));
        } else if key == keybindings.delete {
            assert_eq!(key_to_edit_msg(key, &keybindings), Some(CommonEditMsg::Delete));
        } else if key == keybindings.line_begin || key.keycode == keybindings.home.keycode {
            assert_eq!(
                key_to_edit_msg(key, &keybindings),
                Some(CommonEditMsg::LineBegin {
                    selecting: key.modifiers.shift,
                })
            );
        } else if key == keybindings.line_end {
            assert_eq!(
                key_to_edit_msg(key, &keybindings),
                Some(CommonEditMsg::LineEnd {
                    selecting: key.modifiers.shift,
                })
            );
        } else if key == keybindings.page_up {
            assert_eq!(
                key_to_edit_msg(key, &keybindings),
                Some(CommonEditMsg::PageUp {
                    selecting: key.modifiers.shift,
                })
            );
        } else if key == keybindings.page_down {
            assert_eq!(
                key_to_edit_msg(key, &keybindings),
                Some(CommonEditMsg::PageDown {
                    selecting: key.modifiers.shift,
                })
            );
        } else if key == keybindings.tab {
            assert_eq!(key_to_edit_msg(key, &keybindings), Some(CommonEditMsg::Tab));
        } else if key == keybindings.shift_tab {
            assert_eq!(key_to_edit_msg(key, &keybindings), Some(CommonEditMsg::ShiftTab));
        } else if let Keycode::Char(c) = key.keycode {
            if key.modifiers.is_empty() || key.modifiers.just_shift() {
                assert_eq!(key_to_edit_msg(key, &keybindings), Some(CommonEditMsg::Char(c)));
            } else {
                assert_eq!(key_to_edit_msg(key, &keybindings), None);
            }
        } else {
            assert_eq!(key_to_edit_msg(key, &keybindings), None);
        }
    }
}
