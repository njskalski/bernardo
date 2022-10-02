use std::ops::Range;

use syntect::html::IncludeBackground::No;

use crate::config::theme::Theme;
use crate::io::buffer_output::BufferOutput;
use crate::io::output::Metadata;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::scroll_interpreter::ScrollInterpreter;
use crate::primitives::rect::Rect;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::with_scroll::WithScroll;

pub struct EditorInterpreter<'a> {
    rect: Rect,
    mock_output: &'a MockOutput,

    scroll: Option<ScrollInterpreter<'a>>,
    hover_rect: Option<Rect>,
}

impl<'a> EditorInterpreter<'a> {
    pub fn new(mock_output: &'a MockOutput, rect: Rect) -> Self {
        let scrolls: Vec<&Metadata> = mock_output.get_meta_by_type(WithScroll::<EditorWidget>::TYPENAME).collect();
        debug_assert!(scrolls.len() < 2);
        let scroll: Option<ScrollInterpreter> = if scrolls.is_empty() {
            None
        } else {
            Some(ScrollInterpreter::new(scrolls[0].rect, mock_output))
        };

        let hovers: Vec<Metadata> = mock_output.get_meta_by_type(WithScroll::TYPENAME).collect();
        debug_assert!(scrolls.len() < 2);
        let scroll_rect: Option<Rect> = if scrolls.empty() {
            None
        } else {
            scrolls[0].rect
        };


        Self {
            rect,
            mock_output,
            scroll,

            // hover_rect: None,
        }
    }
}