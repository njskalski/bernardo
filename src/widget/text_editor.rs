use std::borrow::Borrow;
use std::iter::Iterator;

use log::warn;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::io::style::{TextStyle, TextStyle_WhiteOnBlack, TextStyle_WhiteOnBlue, TextStyle_WhiteOnBrightYellow, TextStyle_WhiteOnRedish};
use crate::primitives::arrow::Arrow;
use crate::primitives::cursor_set::{CursorSet, CursorStatus};
use crate::primitives::xy::XY;
use crate::text::buffer::Buffer;
use crate::text::buffer_state::BufferState;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget};

#[derive(Debug)]
enum TextEditorMsg {
    Arrow(Arrow),
    Letter(char),
}

impl AnyMsg for TextEditorMsg {}

pub struct TextEditorWidget {
    id: WID,
    buffer: Box<dyn Buffer>,
    cursor_set: CursorSet,
}

impl TextEditorWidget {
    pub fn new() -> TextEditorWidget {
        TextEditorWidget {
            id: get_new_widget_id(),
            buffer: Box::new(BufferState::new().with_text("aaa\nbbb\nccc\nd")),
            cursor_set: CursorSet::single(),
        }
    }
}

impl Widget for TextEditorWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        "TextEditor"
    }

    fn min_size(&self) -> XY {
        XY::new(12, 7)
    }

    fn layout(&mut self, max_size: XY) -> XY {
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            InputEvent::KeyInput(key) => {
                match key.keycode {
                    Keycode::ArrowUp => Some(Box::new(TextEditorMsg::Arrow(Arrow::Up))),
                    Keycode::ArrowDown => Some(Box::new(TextEditorMsg::Arrow(Arrow::Down))),
                    Keycode::ArrowLeft => Some(Box::new(TextEditorMsg::Arrow(Arrow::Left))),
                    Keycode::ArrowRight => Some(Box::new(TextEditorMsg::Arrow(Arrow::Right))),
                    Keycode::Char(letter) => Some(Box::new(TextEditorMsg::Letter(letter))),
                    _ => None,
                }
            },
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<TextEditorMsg>();
        if our_msg.is_none() {
            warn!("expecetd TextEditorMsg, got {:?}", msg);
            return None;
        }

        return match our_msg.unwrap() {
            TextEditorMsg::Arrow(arrow) => match arrow {
                Arrow::Up => {
                    self.cursor_set.move_vertically_by(self.buffer.borrow(), -1);
                    None
                }
                Arrow::Down => {
                    self.cursor_set.move_vertically_by(self.buffer.borrow(), 1);
                    None
                }
                Arrow::Left => {
                    self.cursor_set.move_left();
                    None
                }
                Arrow::Right => {
                    self.cursor_set.move_right(self.buffer.borrow());
                    None
                }
            },
            TextEditorMsg::Letter(letter) => {
                let mut all_good = true; //TODO send the sound of lost message
                for c in self.cursor_set.iter().rev() {
                    all_good &= self.buffer.insert(c.a, letter.to_string().as_str());
                }
                self.cursor_set.move_right(self.buffer.borrow());
                None
            }
        };
    }

    fn get_focused(&self) -> &dyn Widget {
        self
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        self
    }

    fn render(&self, focused: bool, output: &mut Output) {
        let len_lines = self.buffer.len_lines();
        let len_lines_cols = format!("{}", len_lines).len();

        let numbers_style = TextStyle_WhiteOnBlue;

        for (line_idx, line) in self.buffer.lines().enumerate() {
            let local_len = format!("{}", line_idx).len();
            let prefix = len_lines_cols - local_len;

            for xi in 0..prefix {
                output.print_at(XY::new(xi as u16, line_idx as u16), numbers_style, " ");
            }

            output.print_at(
                XY::new(prefix as u16, line_idx as u16), //TODO
                numbers_style,
                &format!("{} ", line_idx),
            );
            let x_offset = len_lines_cols + 1;

            // the text
            let line_offset = self.buffer.line_to_char(line_idx);

            let mut x = 0;
            for (gi, gh) in line.graphemes(true).enumerate() {
                let char_idx = line_offset + gi;

                let mut style = match self.cursor_set.get_cursor_status_for_char(char_idx) {
                    CursorStatus::None => TextStyle_WhiteOnBlack,
                    CursorStatus::WithinSelection => TextStyle_WhiteOnBrightYellow,
                    CursorStatus::UnderCursor => TextStyle_WhiteOnRedish,
                };

                if gh.starts_with("\n") {
                    style.foreground = style.foreground.half();
                }

                output.print_at(
                    XY::new((x_offset + x) as u16, line_idx as u16),
                    style,
                    if gh.starts_with("\n") { "\\" } else { gh },
                );

                x += gh.width();
            }

            //after the last character, we still need to draw a cursor.

            if line_idx == self.buffer.len_lines() - 1 {
                let char_idx = line_offset + line.len();

                let mut style = match self.cursor_set.get_cursor_status_for_char(char_idx) {
                    CursorStatus::None => TextStyle_WhiteOnBlack,
                    CursorStatus::WithinSelection => TextStyle_WhiteOnBrightYellow,
                    CursorStatus::UnderCursor => TextStyle_WhiteOnRedish,
                };

                style.foreground = style.foreground.half();

                output.print_at(
                    XY::new((x_offset + x) as u16, line_idx as u16),
                    style,
                    ".",
                );
            }
        }
    }
}