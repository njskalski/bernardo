use log::error;

use crate::cursor::cursor::CursorStatus;
use crate::io::buffer_output::buffer_output_consistent_items_iter::BufferConsistentItemsIter;
use crate::io::buffer_output::horizontal_iter_item::{ConsistentHorizontalItem, HorizontalIterItem};
use crate::io::cell::Cell;
use crate::io::output::Metadata;
use crate::io::style::TextStyle;
use crate::mocks::completion_interpreter::CompletionInterpreter;
use crate::mocks::context_menu_interpreter::ContextMenuInterpreter;
use crate::mocks::editbox_interpreter::EditWidgetInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::savefile_interpreter::SaveFileInterpreter;
use crate::mocks::scroll_interpreter::ScrollInterpreter;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widgets::context_menu::widget::CONTEXT_MENU_WIDGET_NAME;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::editor_widget::completion::completion_widget::CompletionWidget;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::save_file_dialog::save_file_dialog::SaveFileDialogWidget;
use crate::widgets::with_scroll::with_scroll::WithScroll;

pub struct EditorInterpreter<'a> {
    meta: &'a Metadata,
    mock_output: &'a MetaOutputFrame,

    rect_without_scroll: Rect,
    scroll: ScrollInterpreter<'a>,
    compeltion_op: Option<CompletionInterpreter<'a>>,

    saveas_op: Option<SaveFileInterpreter<'a>>,

    find_op: Option<EditWidgetInterpreter<'a>>,
    replace_op: Option<EditWidgetInterpreter<'a>>,

    contextbar_op: Option<ContextMenuInterpreter<'a>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct LineIdxPair {
    pub y: u16,
    pub visible_idx: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct LineIdxTuple {
    pub y: u16,
    pub visible_idx: usize,
    pub contents: HorizontalIterItem,
}

impl<'a> EditorInterpreter<'a> {
    pub fn new(mock_output: &'a MetaOutputFrame, meta: &'a Metadata) -> Option<Self> {
        debug_assert!(
            meta.typename != EditorWidget::TYPENAME,
            "this interpreter is NOT compatible with EditorWidget, please pass entire EditorView (I need scroll line numbers)."
        );

        debug_assert!(
            meta.typename == EditorView::TYPENAME,
            "expected TYPENAME {}, got {}",
            EditorView::TYPENAME,
            meta.typename
        );

        let scrolls: Vec<&Metadata> = mock_output
            .get_meta_by_type(WithScroll::<EditorWidget>::TYPENAME_FOR_MARGIN)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        debug_assert!(scrolls.len() < 2);
        let scroll: ScrollInterpreter = if scrolls.is_empty() {
            error!("failed to find scroll, not returning EditorInterpreter!");
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

        let saveases: Vec<&Metadata> = mock_output
            .get_meta_by_type(SaveFileDialogWidget::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();
        debug_assert!(saveases.len() < 2);
        let saveas_op: Option<SaveFileInterpreter> = if saveases.is_empty() {
            None
        } else {
            Some(SaveFileInterpreter::new(saveases[0], mock_output))
        };

        let contextbars: Vec<&Metadata> = mock_output
            .get_meta_by_type(CONTEXT_MENU_WIDGET_NAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();
        debug_assert!(contextbars.len() < 2);
        let contextbar_op: Option<ContextMenuInterpreter> = contextbars.first().map(|c| ContextMenuInterpreter::new(mock_output, c));

        let rect_without_scroll = mock_output.get_meta_by_type(EditorWidget::TYPENAME).next().unwrap().rect;

        let edit_boxes: Vec<&Metadata> = mock_output
            .get_meta_by_type(EditBoxWidget::TYPENAME)
            .filter(|eb| {
                meta.rect.contains_rect(eb.rect)
                    // and is NOT contained by eventual save as
                    && saveas_op.as_ref().map(|s| !s.meta().rect.contains_rect(eb.rect)).unwrap_or(true)
            })
            .collect();

        assert!(edit_boxes.len() <= 2);

        let (find_op, replace_op): (Option<EditWidgetInterpreter>, Option<EditWidgetInterpreter>) = match edit_boxes.len() {
            1 => (Some(EditWidgetInterpreter::new(edit_boxes[0], mock_output)), None),
            2 => (
                Some(EditWidgetInterpreter::new(edit_boxes[0], mock_output)),
                Some(EditWidgetInterpreter::new(edit_boxes[1], mock_output)),
            ),
            _ => (None, None),
        };

        Some(Self {
            meta,
            mock_output,
            rect_without_scroll,
            scroll,
            compeltion_op,
            saveas_op,
            find_op,
            replace_op,
            contextbar_op,
        })
    }

    // returns cursors in SCREEN SPACE
    pub fn get_visible_cursor_cells(&self) -> impl Iterator<Item = (XY, &Cell)> + '_ {
        self.mock_output
            .buffer
            .cells_iter()
            .with_rect(self.rect_without_scroll)
            .filter(|(_pos, cell)| match cell {
                Cell::Begin { style, grapheme: _ } => {
                    let mut cursor_background = self.mock_output.theme.cursor_background(CursorStatus::UnderCursor).unwrap();
                    if !self.is_editor_focused() {
                        cursor_background = cursor_background.half();
                    }

                    style.background == cursor_background
                }
                Cell::Continuation => false,
            })
    }

    pub fn consistent_items_iter(&self) -> BufferConsistentItemsIter {
        self.mock_output.buffer.consistent_items_iter().with_rect(self.rect_without_scroll)
    }

    pub fn get_visible_cursor_line_indices(&self) -> impl Iterator<Item = LineIdxPair> + '_ {
        let offset = self.scroll.lowest_number().unwrap();
        self.get_visible_cursor_cells().map(move |(xy, _)| LineIdxPair {
            y: xy.y,
            visible_idx: xy.y as usize + offset,
        })
    }

    /*
    Return visible items conforming to given style, enriched with information about line index they are displayed in
    TODO: not tested
     */
    pub fn get_indexed_items_by_style(&self, style: TextStyle) -> impl Iterator<Item = LineIdxTuple> + '_ {
        let offset = self.scroll.lowest_number().unwrap();

        self.mock_output
            .buffer
            .items_of_style(style)
            .with_rect(self.rect_without_scroll.clone())
            .map(move |horizontal_iter_item: HorizontalIterItem| {
                assert!(horizontal_iter_item.text_style.is_some());

                LineIdxTuple {
                    y: horizontal_iter_item.absolute_pos.y,
                    visible_idx: horizontal_iter_item.absolute_pos.y as usize + offset,
                    contents: horizontal_iter_item,
                }
            })
    }

    pub fn get_warnings(&self) -> impl Iterator<Item = LineIdxTuple> + '_ {
        self.get_indexed_items_by_style(self.mock_output.theme.editor_label_warning())
    }

    pub fn get_errors(&self) -> impl Iterator<Item = LineIdxTuple> + '_ {
        self.get_indexed_items_by_style(self.mock_output.theme.editor_label_error())
    }

    pub fn get_type_annotations(&self) -> impl Iterator<Item = LineIdxTuple> + '_ {
        self.get_indexed_items_by_style(self.mock_output.theme.editor_label_type_annotation())
    }

    /*
    first item is u16 0-based screen position
    second item is usize 1-based display line idx
    third item is line contents
     */
    pub fn get_visible_cursor_lines(&self) -> impl Iterator<Item = LineIdxTuple> + '_ {
        let offset = self.scroll.lowest_number().unwrap();
        self.get_visible_cursor_cells()
            .map(move |(xy, _)| {
                self.get_line_by_y(xy.y).map(|line| LineIdxTuple {
                    y: xy.y,
                    visible_idx: xy.y as usize + offset,
                    contents: line,
                })
            })
            .flatten()
    }

    pub fn get_line_by_y(&self, screen_pos_y: u16) -> Option<HorizontalIterItem> {
        debug_assert!(self.meta.rect.lower_right().y > screen_pos_y);
        self.mock_output
            .buffer
            .lines_iter()
            .with_rect(self.rect_without_scroll)
            .skip(screen_pos_y as usize)
            .next()
    }

    pub fn get_all_visible_lines(&self) -> impl Iterator<Item = LineIdxTuple> + '_ {
        let offset = self.scroll.lowest_number().unwrap();
        self.mock_output
            .buffer
            .lines_iter()
            .with_rect(self.rect_without_scroll)
            .map(move |line| LineIdxTuple {
                y: line.absolute_pos.y,
                visible_idx: line.absolute_pos.y as usize + offset,
                contents: line,
            })
    }

    pub fn completions(&self) -> Option<&CompletionInterpreter<'a>> {
        self.compeltion_op.as_ref()
    }

    pub fn save_file_dialog(&self) -> Option<&SaveFileInterpreter<'a>> {
        self.saveas_op.as_ref()
    }

    /*
    Returns "coded" cursor lines, where cursor is coded as in cursor tests, so:
    # <- this is cursor
    [ <- this is a left edge of cursor with anchor
    ( <- this is a left edge of cursor with anchor on the opposite edge

    CURRENTLY DOES NOT SUPPORT MULTI LINE CURSORS
    also, this is not properly tested. It's Bullshit and Duct Tape™ quality.
     */
    pub fn get_visible_cursor_lines_with_coded_cursors(&self) -> impl Iterator<Item = LineIdxTuple> + '_ {
        // Setting colors
        let mut under_cursor = self.mock_output.theme.cursor_background(CursorStatus::UnderCursor).unwrap();
        if !self.is_editor_focused() {
            under_cursor = under_cursor.half();
        }
        let mut within_selection = self.mock_output.theme.cursor_background(CursorStatus::WithinSelection).unwrap();
        if !self.is_editor_focused() {
            within_selection = within_selection.half();
        }
        // let mut default = self.mock_output.theme.default_text(self.is_editor_focused()).background;

        // This does not support multi-column chars now
        self.get_visible_cursor_lines().map(move |mut line_idx| {
            let mut first: Option<u16> = None;
            let mut last: Option<u16> = None;
            let mut anchor: Option<u16> = None;

            'line_loop_1: for x in self.rect_without_scroll.pos.x..self.rect_without_scroll.lower_right().x {
                let pos = XY::new(x, line_idx.y);
                let cell = &self.mock_output.buffer[pos];
                match cell {
                    Cell::Begin { style, grapheme } => {
                        if style.background == under_cursor || style.background == within_selection {
                            if first.is_none() {
                                first = Some(x);
                            }
                            last = Some(x);
                        }
                        if style.background == under_cursor {
                            debug_assert!(anchor.is_none());
                            anchor = Some(x);
                        }

                        if grapheme == "⏎" {
                            break 'line_loop_1;
                        }
                    }
                    Cell::Continuation => {}
                }
            }

            debug_assert!(anchor == first || anchor == last);
            let mut result = String::new();

            'line_loop_2: for x in self.rect_without_scroll.pos.x..self.rect_without_scroll.lower_right().x {
                let pos = XY::new(x, line_idx.y);
                let cell = &self.mock_output.buffer[pos];
                match cell {
                    Cell::Begin { style: _, grapheme } => {
                        if Some(x) == first {
                            if first == last {
                                result += "#";
                            } else {
                                result += if first == anchor { "[" } else { "(" };
                            }
                        }

                        if first != last && Some(x) == last && Some(x) == anchor {
                            result += "]";
                        }

                        result += grapheme;

                        if first != last && Some(x) == last && Some(x) != anchor {
                            result += ")";
                        }

                        if grapheme == "⏎" || grapheme == "⇱" {
                            break 'line_loop_2;
                        }
                    }
                    Cell::Continuation => {}
                }
            }

            // debug!("res [{}]", &result);

            line_idx.contents.text = result;
            line_idx
        })
    }

    pub fn is_view_focused(&self) -> bool {
        self.meta.focused
    }

    pub fn is_editor_focused(&self) -> bool {
        self.mock_output
            .get_meta_by_type(EditorWidget::TYPENAME)
            .filter(|meta| self.meta.rect.contains_rect(meta.rect))
            .next()
            .unwrap()
            .focused
    }

    pub fn find_op(&self) -> Option<&EditWidgetInterpreter<'a>> {
        self.find_op.as_ref()
    }

    pub fn replace_op(&self) -> Option<&EditWidgetInterpreter<'a>> {
        self.replace_op.as_ref()
    }

    pub fn context_bar_op(&self) -> Option<&ContextMenuInterpreter<'a>> {
        self.contextbar_op.as_ref()
    }
}
