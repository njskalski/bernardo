use test_log::test;

use crate::experiments::screen_shot::screenshot;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::widgets::tests::editor_view_testbed::EditorViewTestbed;
use crate::widgets::tests::editor_widget_testbed_builder::EditorWidgetTestbedBuilder;

pub fn get_setup() -> EditorViewTestbed {
    let editor_view_testbed = EditorWidgetTestbedBuilder::new().build_editor();

    {
        let some_text = r#"use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("./src");

    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);

    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);

    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);

    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);

    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);

    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);
    println!("{:?}", path);

    // some comment to avoid formatting collapse
}"#;
        let mut buffer_lock = editor_view_testbed.editor_view.get_buffer_ref().lock_rw().unwrap();
        buffer_lock.set_text(some_text);
    }

    editor_view_testbed
}

#[test]
fn editor_view_context() {
    let mut setup = get_setup();
    setup.next_frame();

    for _i in 0..5 {
        setup.push_input(InputEvent::KeyInput(Keycode::ArrowDown.to_key()))
    }

    // let interpreter = setup.interpreter().unwrap();
    // assert_eq!(interpreter.get_visible_cursor_line_indices().next().unwrap().visible_idx, 6);

    screenshot(&setup.frame_op().unwrap().buffer);
}
