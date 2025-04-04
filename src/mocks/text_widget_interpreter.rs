use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::primitives::rect::Rect;
use crate::widgets::text_widget::TextWidget;

#[derive(Clone, Debug)]
pub struct TextWidgetInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,
}

impl<'a> TextWidgetInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        debug_assert!(meta.typename == TextWidget::TYPENAME);

        Self { meta, output }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }

    pub fn rect(&self) -> Rect {
        self.meta.rect
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

    // pub fn cursor_pos(&self) -> usize {
    //     let cursor_bg = self.output.theme.cursor_background(CursorStatus::UnderCursor).unwrap();
    //
    //     self.output
    //         .buffer
    //         .cells_iter()
    //         .find(|(_pos, cell)| match cell {
    //             Cell::Begin { style, .. } => style.background == cursor_bg,
    //             Cell::Continuation => false,
    //         })
    //         .map(|(pos, _cell)| pos.x as usize)
    //         .unwrap()
    // }
}
