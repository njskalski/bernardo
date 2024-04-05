use crate::io::output::Metadata;
use crate::mocks::editor_interpreter::EditorInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::widgets::code_results_view::code_results_widget::CodeResultsView;
use crate::widgets::editor_view::editor_view::EditorView;

pub struct CodeResultsViewInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,

    editors: Vec<EditorInterpreter<'a>>,
}

impl<'a> CodeResultsViewInterpreter<'a> {
    pub fn new(output: &'a MetaOutputFrame, meta: &'a Metadata) -> Self {
        debug_assert!(meta.typename == CodeResultsView::TYPENAME);

        let editors_meta: Vec<&'a Metadata> = output
            // WIDGET not VIEW here
            .get_meta_by_type(EditorView::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        let editors: Vec<EditorInterpreter<'a>> = editors_meta
            .into_iter()
            .map(|editor_meta| EditorInterpreter::new(output, editor_meta).unwrap())
            .collect();

        Self { meta, output, editors }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }

    pub fn editors(&self) -> &Vec<EditorInterpreter<'a>> {
        &self.editors
    }

    // pub fn contents(&self) -> String {
    //     self.output.buffer.lines_iter().with_rect(self.meta.rect).next().unwrap().text.trim().to_string()
    // }
}
