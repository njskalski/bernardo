use crate::io::{input::Input, input_event::InputEvent};
use crate::io::keys::Keycode;

use super::check_box_testbed::{CheckBoxTestbed, CheckBoxTestbedBuilder};

const TEXT: &'static str = "This is example checkbox text.
with
multiple
lines.";

fn get_setup() -> CheckBoxTestbed {
  CheckBoxTestbedBuilder::default().build(TEXT)
}

#[test]
fn click_test() {
  let mut setup = get_setup();
  // setup.next_frame();
  assert!(!setup.widget.is_enabled());
  let enter_input = InputEvent::KeyInput(Keycode::Enter.to_key());
  setup.send_input(enter_input);
  // assert!(setup.widget.is_enabled());
}

fn two_click_test() {
  todo!()
}

fn even_clicks_test() {
  todo!()
}

fn odd_clicks_test() {
  todo!()
}