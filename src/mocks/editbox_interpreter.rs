use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::widgets::edit_box::EditBoxWidget;

pub struct EditWidgetInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,
}

impl<'a> EditWidgetInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        debug_assert!(meta.typename == EditBoxWidget::TYPENAME);

        Self { meta, output }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }

    pub fn contents(&self) -> String {
        self.output
            .buffer
            .lines_iter()
            .with_rect(self.meta.rect)
            .next()
            .unwrap()
            .text
            .trim()
            .to_string()
    }
}
