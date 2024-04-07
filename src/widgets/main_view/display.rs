use crate::widget::widget::Widget;
use crate::widgets::code_results_view::code_results_widget::CodeResultsView;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::main_view::main_view::DocumentIdentifier;

pub enum MainViewDisplay {
    Editor(EditorView),
    ResultsView(CodeResultsView),
}

impl MainViewDisplay {
    pub fn get_widget(&self) -> &dyn Widget {
        match self {
            MainViewDisplay::Editor(e) => e,
            MainViewDisplay::ResultsView(r) => r,
        }
    }

    pub fn get_widget_mut(&mut self) -> &mut dyn Widget {
        match self {
            MainViewDisplay::Editor(e) => e,
            MainViewDisplay::ResultsView(r) => r,
        }
    }

    pub fn get_document_identifier(&self) -> Option<&DocumentIdentifier> {
        match self {
            MainViewDisplay::Editor(editor_view) => Some(editor_view.get_buffer_ref().document_identifier()),
            _ => None,
        }
    }
}
