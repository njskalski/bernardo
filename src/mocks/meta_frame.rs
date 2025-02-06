use crate::config::theme::Theme;
use crate::fs::path::SPath;
use crate::io::buffer_output::buffer_output::BufferOutput;
use crate::io::output::Metadata;
use crate::mocks::code_results_interpreter::CodeResultsViewInterpreter;
use crate::mocks::context_menu_interpreter::ContextMenuInterpreter;
use crate::mocks::editor_interpreter::EditorInterpreter;
use crate::mocks::generic_dialog_interpreter::GenericDialogWidgetInterpreter;
use crate::mocks::nested_menu_interpreter::NestedMenuInterpreter;
use crate::mocks::no_editor_interpreter::NoEditorInterpreter;
use crate::mocks::with_scroll_interpreter::WithScrollWidgetInterpreter;
use crate::widget::widget::Widget;
use crate::widgets::code_results_view::code_results_widget::CodeResultsView;
use crate::widgets::context_menu::widget::{ContextMenuWidget, CONTEXT_MENU_WIDGET_NAME};
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::find_in_files_widget::find_in_files_widget::FindInFilesWidget;
use crate::widgets::find_in_files_widget::tests::find_in_files_widget_interpreter::FindInFilesWidgetInterpreter;
use crate::widgets::generic_dialog::generic_dialog::GenericDialog;
use crate::widgets::nested_menu::widget::NESTED_MENU_TYPENAME;
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::spath_tree_view_node::FileTreeNode;
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

    pub fn get_context_menus(&self) -> impl Iterator<Item = ContextMenuInterpreter> {
        self.get_meta_by_type(CONTEXT_MENU_WIDGET_NAME)
            .map(|meta| ContextMenuInterpreter::new(self, meta))
    }

    pub fn get_first_generic_dialogs(&self) -> Option<GenericDialogWidgetInterpreter<'_>> {
        self.get_meta_by_type(GenericDialog::TYPENAME)
            .map(|meta| GenericDialogWidgetInterpreter::new(meta, self))
            .next()
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

    pub fn get_fuzzy_search(&self) -> Option<ContextMenuInterpreter> {
        self.get_meta_by_type(ContextMenuWidget::<SPath, FileTreeNode>::static_typename())
            .map(|meta| ContextMenuInterpreter::new(self, meta))
            .next()
    }

    pub fn get_code_results_view(&self) -> Option<CodeResultsViewInterpreter> {
        self.get_meta_by_type(CodeResultsView::TYPENAME)
            .map(|meta| CodeResultsViewInterpreter::new(self, meta))
            .next()
    }

    pub fn get_find_in_files(&self) -> Option<FindInFilesWidgetInterpreter> {
        self.get_meta_by_type(FindInFilesWidget::static_typename())
            .map(|meta| FindInFilesWidgetInterpreter::new(meta, self))
            .next()
    }
}
