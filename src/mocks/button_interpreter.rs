use log::debug;

use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_output::MockOutput;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widgets::button::ButtonWidget;
use crate::widgets::edit_box::EditBoxWidget;

#[derive(Clone, Debug)]
pub struct ButtonWidgetInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,
}

impl<'a> ButtonWidgetInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        debug_assert!(meta.typename == ButtonWidget::TYPENAME);

        Self {
            meta,
            output,
        }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }

    pub fn contents(&self) -> String {
        self.output.buffer.lines_iter().with_rect(self.meta.rect).next().unwrap().trim().to_string()
    }
}