use crate::experiments::screen_shot::screenshot;
use crate::mocks::mock_labels_provider::MockLabelsProvider;
use crate::primitives::printable::Printable;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::widgets::editor_widget::label::label::{Label, LabelPos, LabelStyle};
use crate::widgets::editor_widget::label::labels_provider::LabelsProvider;
use crate::widgets::tests::editor_view_testbed::EditorViewTestbed;
use crate::widgets::tests::widget_testbed_builder::WidgetTestbedBuilder;

fn get_setup() -> EditorViewTestbed {
    let mut mock_labels_provider = MockLabelsProvider::new();

    mock_labels_provider.labels.push(
        Label::new(
            LabelPos::Inline { char_idx: 49 },
            LabelStyle::TypeAnnotation,
            Box::new(
                ":PathBuf".to_string()
            )));
    mock_labels_provider.labels.push(
        Label::new(
            LabelPos::LineAfter { line_no_1b: 3 },
            LabelStyle::Warning,
            Box::new("just a warning".to_string()),
        )
    );
    mock_labels_provider.labels.push(
        Label::new(
            LabelPos::InlineStupid { stupid_cursor: StupidCursor { char_idx_0b: 8, line_0b: 5 } },
            LabelStyle::Error,
            Box::new("random error annotation".to_string()),
        )
    );


    let mut editor_view_testbed = WidgetTestbedBuilder::new()
        .with_label_provider(
            mock_labels_provider.into_ref()
        )
        .build_editor();


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
fn editor_label_type_annotation_inline() {
    let mut setup = get_setup();
    setup.next_frame();

    screenshot(&setup.frame_op().as_ref().unwrap().buffer);

    assert!(setup.interpreter().unwrap().is_editor_focused());

    let interpreter = setup.interpreter().unwrap();

    let first_type = interpreter.get_type_annotations().next().unwrap();
    assert_eq!(first_type.y, 3);
    assert_eq!(first_type.contents.text, ":PathBuf");

    assert_eq!(interpreter.get_line_by_y(3).unwrap().text.trim(), "let path:PathBuf = PathBuf::from(\"./src\");⏎");
}

#[test]
fn editor_label_warning_end_of_line() {
    let mut setup = get_setup();
    setup.next_frame();

    assert!(setup.interpreter().unwrap().is_editor_focused());

    let interpreter = setup.interpreter().unwrap();

    let first_type = interpreter.get_warnings().next().unwrap();
    assert_eq!(first_type.y, 2);
    assert_eq!(first_type.contents.text, "just a warning");

    assert_eq!(interpreter.get_line_by_y(2).unwrap().text.trim(), "fn main() {⏎just a warning");
}

#[test]
fn editor_label_error_stupid_cursor() {
    let mut setup = get_setup();
    setup.next_frame();

    assert!(setup.interpreter().unwrap().is_editor_focused());

    let interpreter = setup.interpreter().unwrap();

    let first_type = interpreter.get_errors().next().unwrap();
    assert_eq!(first_type.y, 5);
    assert_eq!(first_type.contents.text, "random error annotation");

    assert_eq!(interpreter.get_line_by_y(5).unwrap().text.trim(), "// srandom error annotationome comment to avoid formatting collapse⏎");
}