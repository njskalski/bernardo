use crate::config::theme::Theme;
use crate::io::buffer_output::BufferOutput;
use crate::io::output::Metadata;
use crate::mocks::editor_interpreter::EditorInterpreter;
use crate::mocks::fuzzy_search_interpreter::FuzzySearchInterpreter;
use crate::mocks::no_editor_interpreter::NoEditorInterpreter;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::fuzzy_search::fuzzy_search::FuzzySearchWidget;
use crate::widgets::no_editor::NoEditorWidget;

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

    pub fn get_no_editor(&self) -> Option<NoEditorInterpreter> {
        self.get_meta_by_type(NoEditorWidget::TYPENAME).map(|meta|
            NoEditorInterpreter::new(self, meta)
        ).next()
    }

    pub fn get_fuzzy_search(&self) -> Option<FuzzySearchInterpreter> {
        self.get_meta_by_type(FuzzySearchWidget::TYPENAME).map(|meta|
            FuzzySearchInterpreter::new(self, meta)
        ).next()
    }
}
