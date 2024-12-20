use crate::io::keys::Keycode;
use crate::io::{input::Input, input_event::InputEvent};

use super::check_box_testbed::{CheckBoxTestbed, CheckBoxTestbedBuilder};

const TEXT: &'static str = "single line text";
// TODO below text causes panick in bernardo::io::buffer::Buffer::flatten_index
// const TEXT: &'static str = "multiple\n line\n text\n";

fn get_setup() -> CheckBoxTestbed {
    CheckBoxTestbedBuilder::default().build(TEXT)
}

#[test]
fn click_test() {
    let mut setup = get_setup();
    setup.next_frame();
    assert!(!setup.widget.is_enabled());
    let enter_input = InputEvent::KeyInput(Keycode::Enter.to_key());
    setup.send_input(enter_input);
    assert!(setup.widget.is_enabled());
    setup.screenshot();
}

#[test]
fn two_click_test() {
    let mut setup = get_setup();
    setup.next_frame();
    assert!(!setup.widget.is_enabled());
    let enter_input = InputEvent::KeyInput(Keycode::Enter.to_key());
    setup.send_input(enter_input);
    assert!(setup.widget.is_enabled());
    setup.send_input(enter_input);
    assert!(!setup.widget.is_enabled());
    setup.screenshot();
}

#[test]
fn even_clicks_test() {
    let mut setup = get_setup();
    setup.next_frame();
    assert!(!setup.widget.is_enabled());
    let enter_input = InputEvent::KeyInput(Keycode::Enter.to_key());
    for _ in 0..16 {
        setup.send_input(enter_input);
    }
    assert!(!setup.widget.is_enabled());
    setup.screenshot();
}
#[test]
fn odd_clicks_test() {
    let mut setup = get_setup();
    setup.next_frame();
    assert!(!setup.widget.is_enabled());
    let enter_input = InputEvent::KeyInput(Keycode::Enter.to_key());
    for _ in 0..17 {
        setup.send_input(enter_input);
    }
    assert!(setup.widget.is_enabled());
    setup.screenshot();
}
