use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::editor_widget::context_bar::widget::ContextBarWidget;

pub struct ContextBarWidgetInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,
}

impl<'a> ContextBarWidgetInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        debug_assert!(meta.typename == ContextBarWidget::TYPENAME);

        Self { meta, output }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }

    pub fn selected_option(&self) -> Option<String> {
        self.output
            .buffer
            .lines_iter()
            .with_rect(self.meta.rect)
            .find(|line| {
                line.text_style
                    .map(|style| style.background == self.output.theme.highlighted(self.meta.focused).background)
                    .unwrap_or(false)
            })
            .map(|line| line.text)
    }

    // pub fn contents(&self) -> String {
    //     self.output.buffer.lines_iter().with_rect(self.meta.rect).next().unwrap().text.trim().to_string()
    // }
}
