use std::ops::Range;

use syntect::html::IncludeBackground::No;

use crate::config::theme::Theme;
use crate::io::buffer_output::BufferOutput;
use crate::io::cell::Cell;
use crate::io::output::Metadata;
use crate::mocks::completion_interpreter::CompletionInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::savefile_interpreter::SaveFileInterpreter;
use crate::mocks::scroll_interpreter::ScrollInterpreter;
use crate::primitives::cursor_set::CursorStatus;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widgets::editor_widget::completion::completion_widget::CompletionWidget;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::save_file_dialog::save_file_dialog::SaveFileDialogWidget;
use crate::widgets::with_scroll::WithScroll;

pub struct EditorInterpreter<'a> {
    meta: &'a Metadata,
    mock_output: &'a MetaOutputFrame,

    rect_without_scroll: Rect,
    scroll: ScrollInterpreter<'a>,
    compeltion_op: Option<CompletionInterpreter<'a>>,

    saveas_op: Option<SaveFileInterpreter<'a>>,
}

pub struct LineIdxPair {
    pub y: u16,
    pub visible_idx: usize,
}

pub struct LineIdxTuple {
    pub y: u16,
    pub visible_idx: usize,
    pub contents: String,
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
            Some(CompletionInterpreter::new(comps[0], mock_output))
        };

        let saveases: Vec<&Metadata> = mock_output.get_meta_by_type(SaveFileDialogWidget::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();
        debug_assert!(saveases.len() < 2);
        let saveas_op: Option<SaveFileInterpreter> = if saveases.is_empty() {
            None
        } else {
            Some(SaveFileInterpreter::new(saveases[0], mock_output))
        };

        let rect_without_scroll = mock_output
            .get_meta_by_type(EditorWidget::TYPENAME)
            .next().unwrap().rect;

        Some(Self {
            meta,
            mock_output,
            rect_without_scroll,
            scroll,
            compeltion_op,
            saveas_op,
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

    /*
    first item is u16 0-based screen position
    second item is usize 1-based display line idx
     */
    pub fn get_visible_cursor_line_indices(&self) -> impl Iterator<Item=LineIdxPair> + '_ {
        let offset = self.scroll.lowest_number().unwrap();
        self.get_visible_cursor_cells().map(move |(xy, _)| LineIdxPair { y: xy.y, visible_idx: xy.y as usize + offset })
    }

    /*
    first item is u16 0-based screen position
    second item is usize 1-based display line idx
    third item is line contents
     */
    pub fn get_visible_cursor_lines(&self) -> impl Iterator<Item=LineIdxTuple> + '_ {
        let offset = self.scroll.lowest_number().unwrap();
        self.get_visible_cursor_cells().map(move |(xy, _)| LineIdxTuple {
            y: xy.y,
            visible_idx: xy.y as usize + offset,
            contents: self.get_line_by_y(xy.y).unwrap(),
        })
    }

    pub fn get_line_by_y(&self, screen_pos_y: u16) -> Option<String> {
        debug_assert!(self.meta.rect.lower_right().y > screen_pos_y);
        self.mock_output.buffer.lines_iter().with_rect(self.rect_without_scroll).skip(screen_pos_y as usize).next()
    }

    pub fn completions(&self) -> Option<&CompletionInterpreter<'a>> {
        self.compeltion_op.as_ref()
    }

    pub fn save_file_dialog(&self) -> Option<&SaveFileInterpreter<'a>> { self.saveas_op.as_ref() }

    /*
    Returns "coded" cursor lines, where cursor is coded as in cursor tests, so:
    # <- this is cursor
    [ <- this is a left edge of cursor with anchor
    ( <- this is a left edge of cursor with anchor on the opposite edge

    CURRENTLY DOES NOT SUPPORT MULTI LINE CURSORS
    also, this is not properly tested. It's Bullshit and Duct Tape™ quality.
     */
    pub fn get_visible_coded_cursor_lines(&self) -> impl Iterator<Item=LineIdxTuple> + '_ {
        self.get_visible_cursor_lines().map(|mut line_idx| {
            let mut result = String::new();
            let mut cursor_open = false;
            let mut prev_within_sel = false;
            let mut was_more_than_anchor = false;

            'line_loop:
            for x in self.rect_without_scroll.pos.x..self.rect_without_scroll.lower_right().x {
                let pos = XY::new(x, line_idx.y);
                let cell = &self.mock_output.buffer[pos];
                let mut grapheme_added = false;
                match cell {
                    Cell::Begin { style, grapheme } => {
                        if style.background == self.mock_output.theme.cursor_background(CursorStatus::UnderCursor).unwrap() {
                            if !cursor_open {
                                result += "[";
                                cursor_open = true;
                            } else {
                                result += grapheme;
                                grapheme_added = true;
                                result += "]";
                                cursor_open = false;
                            }
                        }
                        let now_within_sel = style.background == self.mock_output.theme.cursor_background(CursorStatus::WithinSelection).unwrap();
                        if now_within_sel {
                            was_more_than_anchor = true;
                        }

                        if !prev_within_sel && now_within_sel {
                            result += "(";
                        }
                        if prev_within_sel && !now_within_sel {
                            result += ")";
                        }
                        prev_within_sel = now_within_sel;

                        if !grapheme_added {
                            result += grapheme;
                            grapheme_added = true;
                        }

                        if grapheme == "⏎" {
                            break 'line_loop;
                        }
                    }
                    Cell::Continuation => {}
                }
            }

            if !was_more_than_anchor {
                result = result.replace("[", "#");
            }

            line_idx.contents = result;
            line_idx
        })
    }
}