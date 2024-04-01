use crate::mocks::editor_interpreter::EditorInterpreter;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::tests::generic_widget_testbed::GenericWidgetTestbed;

pub type EditorViewTestbed = GenericWidgetTestbed<EditorView>;

impl EditorViewTestbed {
    pub fn interpreter(&self) -> Option<EditorInterpreter<'_>> {
        self.frame_op()
            .and_then(|frame| EditorInterpreter::new(frame, frame.metadata.first().unwrap()))
    }
}
