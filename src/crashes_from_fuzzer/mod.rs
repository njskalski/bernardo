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

#[test]
fn crash5_test() {
    let inputs = vec![
        Key {
            keycode: Home,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('s'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: Char('j'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: Home,
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
fn crash6_test() {
    let inputs = vec![
        Key {
            keycode: Char('a'),
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
            keycode: Char('a'),
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
            keycode: Char('a'),
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
            keycode: Char('a'),
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
            keycode: Char('n'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('n'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('w'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: ArrowUp,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
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
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: ArrowUp,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
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
            keycode: Home,
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
fn crash7_test() {
    let inputs = vec![
        Key {
            keycode: Char('n'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: Char('n'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: Space,
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
                shift: true,
            },
        },
        Key {
            keycode: Char('g'),
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
fn crash8_test() {
    let inputs = vec![
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
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
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('g'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
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
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: LeftCtrl,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: Char('x'),
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
            keycode: Char('a'),
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
            keycode: Char('a'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
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
fn crash9_test() {
    let inputs = vec![
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
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
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('g'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
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
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: LeftCtrl,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: Char('x'),
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
            keycode: Char('a'),
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
            keycode: Char('a'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
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
fn crash10_test() {
    let inputs = vec![
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
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
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('g'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
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
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
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
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('g'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
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
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('g'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: End,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
    ];
    fuzz_call(inputs);
}

#[test]
fn crash11_test() {
    let inputs = vec![
        Key {
            keycode: Char('j'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: ArrowDown,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: ArrowDown,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: ArrowDown,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('m'),
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
fn crash12_test() {
    let inputs = vec![
        Key {
            keycode: Home,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('s'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: Char('j'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
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
            keycode: Char('i'),
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
        Key {
            keycode: Char('j'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
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
        Key {
            keycode: Char('m'),
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
fn crash13_test() {
    let inputs = vec![
        Key {
            keycode: Home,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('s'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: Char('j'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
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
            keycode: Char('i'),
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
        Key {
            keycode: Char('j'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: Char('m'),
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
fn crash14_test() {
    let inputs = vec![
        Key {
            keycode: Char('a'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('g'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
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
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('g'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
    ];
    fuzz_call(inputs);
}

#[test]
fn crash15_test() {
    let inputs = vec![
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('b'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: RightCtrl,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: Char('y'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('s'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: Char('b'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('b'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: RightCtrl,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
        Key {
            keycode: Char('y'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('s'),
            modifiers: Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
            },
        },
        Key {
            keycode: Char('b'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('q'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('q'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Enter,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('q'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: Char('b'),
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: false,
            },
        },
        Key {
            keycode: RightCtrl,
            modifiers: Modifiers {
                alt: false,
                ctrl: false,
                shift: true,
            },
        },
    ];
    fuzz_call(inputs);
}
