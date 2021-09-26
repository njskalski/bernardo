use crate::widget::widget::{Widget, WID, get_new_widget_id};
use crate::primitives::xy::XY;
use crate::io::input_event::InputEvent;
use crate::widget::any_msg::AnyMsg;
use crate::io::output::Output;
use crate::text::buffer_state::BufferState;
use crate::io::style::{TextStyle, TextStyle_WhiteOnBlue, TextStyle_WhiteOnBlack};
use crate::text::buffer::Buffer;

pub struct TextEditorWidget {
    id : WID,
    buffer : Box<dyn Buffer>,
}

impl TextEditorWidget {
    pub fn new() -> TextEditorWidget {
        TextEditorWidget {
            id : get_new_widget_id(),
            buffer : Box::new(BufferState::new()
                .with_text("aaa\nbbb\nccc\nd"))
            ,
        }
    }

}

impl Widget for TextEditorWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn min_size(&self) -> XY {
        XY::new(12, 7)
    }

    fn size(&self, max_size: XY) -> XY {
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
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
        let text_style = TextStyle_WhiteOnBlack;

        for (y, line) in self.buffer.lines().enumerate() {
            let local_len = format!("{}", y).len();
            let prefix = len_lines_cols - local_len;

            for xi in 0..prefix {
                output.print_at(XY::new(xi as u16, y as u16), numbers_style, " ");
            }

            output.print_at(
                XY::new(prefix as u16, y as u16), //TODO
                numbers_style,
                &format!("{} ", y)
            );
            let x_offset = len_lines_cols + 1;

            output.print_at(
                XY::new(x_offset as u16, y as u16),
                text_style,
                line,
            );
        }
    }
}