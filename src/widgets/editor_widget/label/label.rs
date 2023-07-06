use std::ops::Range;

use log::warn;

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::cursor::cursor::{Cursor, NEWLINE_WIDTH, Selection};
use crate::io::style::TextStyle;
use crate::primitives::printable::Printable;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::primitives::styled_printable::{StyleBorrowedPrintable, StyledPrintable, StyleWrappedPrintable};
use crate::primitives::xy::XY;
use crate::text::text_buffer::TextBuffer;
use crate::unpack_or;

pub enum LabelPos {
    /*
    Appears immediately after anchoring symbol, can be cursor selected for context
     */
    Inline { char_idx: usize },
    /*
        both line and column are 1-based
     */
    InlineStupid { stupid_cursor: StupidCursor },

    /*
    Appears above indexed line (1-based)
     */
    // LineAbove { line_no_1b: usize },
    /*
    Appears below indexed line (1-based)
     */
    // LineBelow { line_no_1b: usize },

    /*
    Appears after indexed line (1-based)
     */
    LineAfter { line_no_1b: usize },
}

impl LabelPos {
    /*
    Can return false positives (for now).
     */
    pub fn maybe_should_draw(&self, cursor_range: Range<usize>, line_range: Range<usize>) -> bool {
        match self {
            LabelPos::Inline { char_idx } => {
                cursor_range.contains(char_idx)
            }
            LabelPos::InlineStupid { stupid_cursor } => {
                line_range.contains(&(stupid_cursor.line_0b as usize))
            }
            LabelPos::LineAfter { line_no_1b } => {
                line_range.contains(line_no_1b)
            }
        }
    }

    pub fn into_position(&self, text_buffer: &dyn TextBuffer) -> Option<XY> {
        match self {
            LabelPos::Inline { char_idx } => {
                if let Some(line_no_0b) = text_buffer.char_to_line(*char_idx) {
                    debug_assert!(line_no_0b <= *char_idx);
                    let line_begin_char_idx_0b = text_buffer.line_to_char(line_no_0b)?;
                    let in_line_char_idx = char_idx - line_begin_char_idx_0b;

                    debug_assert!(line_no_0b <= u16::MAX as usize);
                    debug_assert!(in_line_char_idx <= u16::MAX as usize);

                    Some(XY::new(in_line_char_idx as u16, line_no_0b as u16))
                } else {
                    None
                }
            }
            LabelPos::InlineStupid { stupid_cursor } => {
                stupid_cursor.to_xy(text_buffer)
            }
            LabelPos::LineAfter { line_no_1b } => {
                debug_assert!(*line_no_1b >= 1);
                if text_buffer.len_lines() + 1 > *line_no_1b {
                    let line_no_0b = line_no_1b - 1;
                    if line_no_0b > u16::MAX as usize {
                        warn!("line too far");
                        return None;
                    }

                    let line = unpack_or!(text_buffer.get_line(line_no_0b), None);

                    // I add NEWLINE_WIDTH to cover the "⏎" char
                    Some(XY::new(line.screen_width() + NEWLINE_WIDTH, line_no_0b as u16))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum LabelStyle {
    Warning,
    Error,
    TypeAnnotation,
    Random(TextStyle),
}

pub struct Label {
    // TODO make private
    pub pos: LabelPos,
    pub style: LabelStyle,
    contents: Box<dyn Printable + Sync + Send>,
}

impl Label {
    pub fn new(label_pos: LabelPos, style: LabelStyle, contents: Box<dyn Printable + Sync + Send>) -> Self {
        Label {
            pos: label_pos,
            style,
            contents,
        }
    }

    pub fn screen_width(&self) -> u16 {
        self.contents.screen_width()
    }

    pub fn contents(&self, theme: &Theme) -> impl StyledPrintable + '_ {
        let computed_style = match self.style {
            LabelStyle::Warning => {
                theme.ui.label_warning.clone()
            }
            LabelStyle::Error => {
                theme.ui.label_error.clone()
            }
            LabelStyle::TypeAnnotation => {
                theme.ui.label_type_annotation.clone()
            }
            LabelStyle::Random(style) => {
                style
            }
        };

        StyleBorrowedPrintable::new(computed_style, self.contents.as_ref())
    }
}


