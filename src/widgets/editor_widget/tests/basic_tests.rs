use crate::widgets::tests::editor_view_testbed::EditorViewTestbed;
use crate::widgets::tests::widget_testbed_builder::WidgetTestbedBuilder;

pub fn get_setup() -> EditorViewTestbed {
    let mut editor_view_testbed = WidgetTestbedBuilder::new().build_editor();

    {
        let some_text = r#"use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("./src");

    // some comment to avoid formatting collapse
}"#;
        let mut buffer_lock = editor_view_testbed.editor_view.get_buffer_ref().lock_rw().unwrap();
        buffer_lock.set_text(some_text);
    }

    editor_view_testbed
}

#[test]
fn basic_editor_testbed_test() {
    let mut setup = get_setup();
    setup.next_frame();

    assert!(setup.interpreter().unwrap().is_editor_focused());

    {
        // let interpreter = setup.interpreter()?;
    }

    // screenshot(i);
}
