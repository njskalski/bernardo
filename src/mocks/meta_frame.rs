use crate::config::theme::Theme;
use crate::io::buffer_output::BufferOutput;
use crate::io::output::Metadata;
use crate::mocks::editor_interpreter::EditorInterpreter;
use crate::widgets::editor_view::editor_view::EditorView;

/*
No time to come up with good name. It's basically a frame with "metadata" that was emited while it
was rendered.
 */
#[derive(Clone, Debug)]
pub struct MetaOutputFrame {
    pub buffer: BufferOutput,
    pub metadata: Vec<Metadata>,
    pub theme: Theme,
}

impl MetaOutputFrame {
    pub fn get_meta_by_type(&self, typename: &'static str) -> impl Iterator<Item=&Metadata> {
        self.metadata.iter().filter(move |i| i.typename == typename)
    }

    pub fn get_editors(&self) -> impl Iterator<Item=EditorInterpreter> {
        self.get_meta_by_type(EditorView::TYPENAME).map(|meta|
            EditorInterpreter::new(self, meta)
        ).flatten()
    }
}
