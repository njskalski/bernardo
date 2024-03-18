use crate::config::theme::Theme;
use crate::io::buffer_output::buffer_output::BufferOutput;
use crate::io::output::Metadata;
use crate::mocks::code_results_interpreter::CodeResultsViewInterpreter;
use crate::mocks::editor_interpreter::EditorInterpreter;
use crate::mocks::fuzzy_search_interpreter::FuzzySearchInterpreter;
use crate::mocks::nested_menu_interpreter::NestedMenuInterpreter;
use crate::mocks::no_editor_interpreter::NoEditorInterpreter;
use crate::mocks::with_scroll_interpreter::WithScrollWidgetInterpreter;
use crate::widget::widget::Widget;
use crate::widgets::code_results_view::code_results_widget::CodeResultsView;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::fuzzy_search::fuzzy_search::FuzzySearchWidget;
use crate::widgets::nested_menu::widget::{NestedMenuWidget, NESTED_MENU_TYPENAME};
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::with_scroll::with_scroll::WithScroll;

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
    pub fn get_meta_by_type(&self, typename: &'static str) -> impl Iterator<Item = &Metadata> {
        self.metadata.iter().filter(move |i| i.typename == typename)
    }

    pub fn get_editors(&self) -> impl Iterator<Item = EditorInterpreter> {
        self.get_meta_by_type(EditorView::TYPENAME)
            .map(|meta| EditorInterpreter::new(self, meta))
            .flatten()
    }

    pub fn get_nested_menus(&self) -> impl Iterator<Item = NestedMenuInterpreter> {
        self.get_meta_by_type(NESTED_MENU_TYPENAME)
            .map(|meta| NestedMenuInterpreter::new(self, meta))
            .flatten()
    }

    pub fn get_scroll<T: Widget>(&self) -> impl Iterator<Item = WithScrollWidgetInterpreter<T>> {
        self.get_meta_by_type(WithScroll::<T>::TYPENAME_FOR_MARGIN)
            .map(|meta| WithScrollWidgetInterpreter::new(self, meta))
    }

    pub fn get_no_editor(&self) -> Option<NoEditorInterpreter> {
        self.get_meta_by_type(NoEditorWidget::TYPENAME)
            .map(|meta| NoEditorInterpreter::new(self, meta))
            .next()
    }

    pub fn get_fuzzy_search(&self) -> Option<FuzzySearchInterpreter> {
        self.get_meta_by_type(FuzzySearchWidget::TYPENAME)
            .map(|meta| FuzzySearchInterpreter::new(self, meta))
            .next()
    }

    pub fn get_code_results_view(&self) -> Option<CodeResultsViewInterpreter> {
        self.get_meta_by_type(CodeResultsView::TYPENAME)
            .map(|meta| CodeResultsViewInterpreter::new(self, meta))
            .next()
    }
}
