use std::panic;

use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::{Key, Keycode, Modifiers};
use crate::widget::widget::Widget;
use crate::widgets::check_box::CheckBoxWidget;
use crate::{io::input_event::InputEvent, widgets::text_widget::TextWidget};

use super::check_box_testbed::{CheckBoxTestbed, CheckBoxTestbedBuilder};

const TEXT: &'static str = "single line text";
const MULTILINE_TEXT: &'static str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaa\n
fwfwefwefwfw\nwfwfwfwfwf\nwffffffffffffffffff";

fn get_setup() -> CheckBoxTestbed {
    CheckBoxTestbedBuilder::default().build(TEXT)
}

#[test]
fn simple_test_check_box_click() {
    let label = TextWidget::new(Box::new("Test Checkbox".to_string()));
    let mut checkbox = CheckBoxWidget::new(label);
    assert_eq!(checkbox.is_checked(), false);
    let key_event = Key {
        keycode: Keycode::Enter,
        modifiers: Modifiers::default(),
    };
    let input_event = KeyInput(key_event);
    if let Some(msg) = checkbox.on_input(input_event) {
        checkbox.update(msg);
    }

    assert_eq!(checkbox.is_checked(), true);

    if let Some(msg) = checkbox.on_input(input_event) {
        checkbox.update(msg);
    }

    assert_eq!(checkbox.is_checked(), false);
}

#[test]
fn click_test() {
    let mut setup = get_setup();
    setup.next_frame();
    assert!(!setup.widget.is_checked());
    let enter_input = InputEvent::KeyInput(Keycode::Enter.to_key());
    setup.send_input(enter_input);
    assert!(setup.widget.is_checked());
    setup.screenshot();
}

#[test]
fn two_click_test() {
    let mut setup = get_setup();
    setup.next_frame();
    assert!(!setup.widget.is_checked());
    let enter_input = InputEvent::KeyInput(Keycode::Enter.to_key());
    setup.send_input(enter_input);
    assert!(setup.widget.is_checked());
    setup.send_input(enter_input);
    assert!(!setup.widget.is_checked());
    setup.screenshot();
}

#[test]
fn even_clicks_test() {
    let mut setup = get_setup();
    setup.next_frame();
    assert!(!setup.widget.is_checked());
    let enter_input = InputEvent::KeyInput(Keycode::Enter.to_key());
    for _ in 0..16 {
        setup.send_input(enter_input);
    }
    assert!(!setup.widget.is_checked());
    setup.screenshot();
}
#[test]
fn odd_clicks_test() {
    let mut setup = get_setup();
    setup.next_frame();
    assert!(!setup.widget.is_checked());
    let enter_input = InputEvent::KeyInput(Keycode::Enter.to_key());
    for _ in 0..17 {
        setup.send_input(enter_input);
    }
    assert!(setup.widget.is_checked());
    setup.screenshot();
}

#[test]
fn with_checked_test() {
    let mut setup = get_setup();
    setup.next_frame();
    setup.widget = setup.widget.with_checked(true);
    assert!(setup.widget.is_checked());
    let enter_input = InputEvent::KeyInput(Keycode::Enter.to_key());
    setup.send_input(enter_input);
    assert!(!setup.widget.is_checked());
}

#[test]
fn multiline_label_test() {
    let result = panic::catch_unwind(|| {
        let mut setup = CheckBoxTestbedBuilder::default().build(MULTILINE_TEXT);
        setup.next_frame();
        let enter_input = InputEvent::KeyInput(Keycode::Enter.to_key());
        setup.send_input(enter_input);
        assert!(setup.widget.is_checked());
        setup.send_input(enter_input);
        assert!(!setup.widget.is_checked());
        setup.screenshot();
    });
    assert!(
        result.is_ok(),
        "TEST_FAIL \n*\n*\n*\n Test panicked with multiline label \n*\n*\n*\nTEST_FAIL"
    );
}
