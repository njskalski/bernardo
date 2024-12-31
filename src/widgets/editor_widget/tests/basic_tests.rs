use crate::widgets::editor_view::test::editor_view_testbed::EditorViewTestbed;
use crate::widgets::editor_view::test::editor_view_testbed_builder::EditorViewTestbedBuilder;

pub fn get_setup() -> EditorViewTestbed {
    let editor_view_testbed = EditorViewTestbedBuilder::default().build();

    {
        let some_text = r#"use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("./src");

    // some comment to avoid formatting collapse
}"#;
        let mut buffer_lock = editor_view_testbed.widget().get_buffer_ref().lock_rw().unwrap();
        buffer_lock.set_text(some_text);
    }

    editor_view_testbed
}

#[test]
fn basic_editor_testbed_test() {
    let mut setup = get_setup();
    setup.next_frame();

    assert!(setup.interpreter().unwrap().is_editor_focused());

    // screenshot(i);
}
