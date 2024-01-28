use test_log::test;

use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;

use super::edit_box_testbed::EditBoxTestbed;

const TEXT: &'static str = "with single line of text";

fn get_setup(text: &str) -> EditBoxTestbed {
    let mut testbed = EditBoxTestbed::new();
    testbed.widget.set_text(text);
    testbed
}

#[test]
fn arrow_cursor_movement() {
    let mut setup = get_setup(TEXT);
    setup.next_frame();

    assert_eq!(setup.interpreter().contents(), TEXT);
    assert_eq!(setup.interpreter().cursor_pos(), 0);

    let key_left = InputEvent::KeyInput(Keycode::ArrowLeft.to_key());
    let key_right = InputEvent::KeyInput(Keycode::ArrowRight.to_key());

    // Go to end of text by pressing right
    for i in 0..TEXT.len() {
        setup.send_input(key_right);
        assert_eq!(setup.interpreter().cursor_pos(), i + 1);
    }

    // Cursor should be at end now
    assert_eq!(setup.interpreter().cursor_pos(), TEXT.len());

    // Pressing right when cursor is at end shouldn't change position
    setup.send_input(key_right);
    assert_eq!(setup.interpreter().cursor_pos(), TEXT.len());

    // Go to start of text by pressing left
    for i in (0..TEXT.len()).rev() {
        setup.send_input(key_left);
        assert_eq!(setup.interpreter().cursor_pos(), i);
    }

    // Cursor should be at start now
    assert_eq!(setup.interpreter().cursor_pos(), 0);

    // Pressing left when cursor is at start shouldn't change position
    setup.send_input(key_left);
    assert_eq!(setup.interpreter().cursor_pos(), 0);
}

#[test]
fn word_cursor_movement() {
    let mut setup = get_setup(TEXT);
    setup.next_frame();

    assert_eq!(setup.interpreter().contents(), TEXT);
    assert_eq!(setup.interpreter().cursor_pos(), 0);

    let key_ctrl_right = InputEvent::KeyInput(Keycode::ArrowRight.to_key().with_ctrl());
    let key_ctrl_left = InputEvent::KeyInput(Keycode::ArrowLeft.to_key().with_ctrl());

    // The cursor will move in the following sequence with ctrl-left/right
    // with single line of text
    // ^   ^^     ^^   ^^ ^^  ^
    let word_mov_pos = [4, 5, 11, 12, 16, 17, 19, 20];

    // Move to end with ctrl-right while checking intermediate positions
    for pos in word_mov_pos {
        setup.send_input(key_ctrl_right);
        assert_eq!(setup.interpreter().cursor_pos(), pos);
    }

    setup.send_input(key_ctrl_right);
    // Cursor should be at end now
    assert_eq!(setup.interpreter().cursor_pos(), TEXT.len());

    // Pressing ctrl-right when cursor is at end shouldn't change position
    setup.send_input(key_ctrl_right);
    assert_eq!(setup.interpreter().cursor_pos(), TEXT.len());

    // Move to start with ctrl-left while checking intermediate positions
    for pos in word_mov_pos.into_iter().rev() {
        setup.send_input(key_ctrl_left);
        assert_eq!(setup.interpreter().cursor_pos(), pos);
    }

    setup.send_input(key_ctrl_left);
    // Cursor should be at start now
    assert_eq!(setup.interpreter().cursor_pos(), 0);

    // Pressing ctrl-left when cursor is at start shouldn't change position
    setup.send_input(key_ctrl_left);
    assert_eq!(setup.interpreter().cursor_pos(), 0);
}

#[test]
fn addition_deletion() {
    let mut setup = get_setup(TEXT);
    setup.next_frame();

    assert_eq!(setup.interpreter().contents(), TEXT);
    assert_eq!(setup.interpreter().cursor_pos(), 0);

    let key_backspace = InputEvent::KeyInput(Keycode::Backspace.to_key());
    let key_delete = InputEvent::KeyInput(Keycode::Delete.to_key());

    const ADDED: &'static str = "added";

    // Enter characters
    for (i, c) in ADDED.chars().enumerate() {
        setup.send_input(InputEvent::KeyInput(Keycode::Char(c).to_key()));
        assert_eq!(setup.interpreter().cursor_pos(), i + 1);
    }
    assert_eq!(setup.interpreter().contents(), format!("{ADDED}{TEXT}"));
    assert_eq!(setup.interpreter().cursor_pos(), ADDED.len());

    // Delete previously entered characters using Backspace
    for _ in 0..ADDED.len() {
        setup.send_input(key_backspace);
    }
    assert_eq!(setup.interpreter().contents(), TEXT);
    assert_eq!(setup.interpreter().cursor_pos(), 0);

    // Clear the buffer by using Delete repeatedly
    for _ in 0..TEXT.len() {
        setup.send_input(key_delete);
    }
    assert_eq!(setup.interpreter().contents(), "");
    assert_eq!(setup.interpreter().cursor_pos(), 0);
}
