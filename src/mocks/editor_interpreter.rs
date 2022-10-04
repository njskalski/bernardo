use std::ops::Range;

use syntect::html::IncludeBackground::No;

use crate::config::theme::Theme;
use crate::io::buffer_output::BufferOutput;
use crate::io::cell::Cell;
use crate::io::output::Metadata;
use crate::mocks::completion_interpreter::CompletionInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::scroll_interpreter::ScrollInterpreter;
use crate::primitives::cursor_set::CursorStatus;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widgets::editor_widget::completion::completion_widget::CompletionWidget;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::with_scroll::WithScroll;

pub struct EditorInterpreter<'a> {
    meta: &'a Metadata,
    mock_output: &'a MetaOutputFrame,

    scroll: ScrollInterpreter<'a>,
    compeltion_op: Option<CompletionInterpreter<'a>>,
}

impl<'a> EditorInterpreter<'a> {
    pub fn new(mock_output: &'a MetaOutputFrame, meta: &'a Metadata) -> Option<Self> {
        let scrolls: Vec<&Metadata> = mock_output
            .get_meta_by_type(WithScroll::<EditorWidget>::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        debug_assert!(scrolls.len() < 2);
        let scroll: ScrollInterpreter = if scrolls.is_empty() {
            return None;
        } else {
            ScrollInterpreter::new(scrolls[0].rect, mock_output)
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


        Some(Self {
            meta,
            mock_output,
            scroll,
            compeltion_op,
        })
    }

    // returns cursors in SCREEN SPACE
    pub fn get_visible_cursor_cells(&self) -> impl Iterator<Item=(XY, &Cell)> + '_ {
        self.mock_output.buffer.cells_iter().filter(|(pos, cell)|
            match cell {
                Cell::Begin { style, grapheme } => style.background == self.mock_output.theme.cursor_background(CursorStatus::UnderCursor).unwrap(),
                Cell::Continuation => false,
            }
        )
    }

    pub fn get_visible_cursor_line_indices(&self) -> impl Iterator<Item=(usize)> + '_ {
        let offset = self.scroll.lowest_number().unwrap();
        self.get_visible_cursor_cells().map(move |(xy, _)| xy.y as usize + offset)
    }
}