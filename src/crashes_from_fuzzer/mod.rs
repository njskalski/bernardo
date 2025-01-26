use crate::io::keys::Keycode::*;
use crate::io::keys::{Key, Modifiers};
use crate::mocks::fuzz_call::fuzz_call;

#[test]
fn crash1_test() {
    let inputs: Vec<Key> = vec![
        Key {
            keycode: Tab,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Tab,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('z'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: Char('h'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
    ];
    fuzz_call(inputs);
}

#[test]
fn crash2_test() {
    let inputs = vec![
        Key {
            keycode: Char('k'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('k'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Backspace,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
    ];
    fuzz_call(inputs);
}

#[test]
fn crash3_test() {
    let inputs = vec![
        Key {
            keycode: Char('p'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: RightAlt,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: Char('p'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
    ];
    fuzz_call(inputs);
}

#[test]
fn crash4_test() {
    let inputs = vec![
        Key {
            keycode: Tab,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Home,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: Tab,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Home,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: Tab,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('t'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('a'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Tab,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
    ];
    fuzz_call(inputs);
}
