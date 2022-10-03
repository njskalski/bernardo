use std::ops::Range;

use syntect::html::IncludeBackground::No;

use crate::config::theme::Theme;
use crate::io::buffer_output::BufferOutput;
use crate::io::output::Metadata;
use crate::mocks::completion_interpreter::CompletionInterpreter;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::scroll_interpreter::ScrollInterpreter;
use crate::primitives::rect::Rect;
use crate::widgets::editor_widget::completion::completion_widget::CompletionWidget;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::with_scroll::WithScroll;

pub struct EditorInterpreter<'a> {
    meta: &'a Metadata,
    mock_output: &'a MockOutput,

    scroll: Option<ScrollInterpreter<'a>>,
    compeltion_op: Option<CompletionInterpreter<'a>>,
}

impl<'a> EditorInterpreter<'a> {
    pub fn new(mock_output: &'a MockOutput, meta: &'a Metadata) -> Self {
        let scrolls: Vec<&Metadata> = mock_output
            .get_meta_by_type(WithScroll::<EditorWidget>::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        debug_assert!(scrolls.len() < 2);
        let scroll: Option<ScrollInterpreter> = if scrolls.is_empty() {
            None
        } else {
            Some(ScrollInterpreter::new(scrolls[0].rect, mock_output))
        };

        let comps: Vec<&Metadata> = mock_output
            .get_meta_by_type(CompletionWidget::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();
        debug_assert!(comps.len() < 2);
        let compeltion_op: Option<CompletionInterpreter> = if comps.is_empty() {
            None
        } else {
            Some(CompletionInterpreter::new(comps[0].rect, mock_output))
        };


        Self {
            meta,
            mock_output,
            scroll,
            compeltion_op,
        }
    }
}