use log::warn;
use ropey::Rope;

use crate::{AnyMsg, InputEvent, Output, SizeConstraint, Theme, Widget};
use crate::primitives::cursor_set::{CursorSet, CursorStatus};
use crate::primitives::xy::XY;
use crate::text::buffer::Buffer;
use crate::text::buffer_state::BufferState;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::common_edit_msgs::{apply_cme, CommonEditMsg, key_to_edit_msg};
use crate::widgets::editor_view::msg::EditorViewMsg;

const MIN_EDITOR_SIZE: XY = XY::new(32, 10);

const NEWLINE: &'static str = "⏎";
const BEYOND: &'static str = "⇱";

pub struct EditorView {
    wid: WID,
    cursors: CursorSet,

    todo_text: BufferState,
}

impl EditorView {
    pub fn new() -> EditorView {
        EditorView {
            wid: get_new_widget_id(),
            cursors: CursorSet::single(),
            todo_text: BufferState::new(),
        }
    }
}

impl Widget for EditorView {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "editor_view"
    }

    fn min_size(&self) -> XY {
        MIN_EDITOR_SIZE
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        sc.hint().size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            InputEvent::Tick => None,
            InputEvent::KeyInput(key) => {
                match key_to_edit_msg(key) {
                    None => None,
                    Some(edit_msg) => Some(Box::new(EditorViewMsg::EditMsg(edit_msg)))
                }
            }
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        return match msg.as_msg::<EditorViewMsg>() {
            None => {
                warn!("expecetd EditorViewMsg, got {:?}", msg);
                None
            }
            Some(msg) => match msg {
                EditorViewMsg::EditMsg(cem) => {
                    let _noop = apply_cme(*cem, &mut self.cursors, &mut self.todo_text);
                    None
                }
                _ => {
                    warn!("unhandled message {:?}", msg);
                    None
                }
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        for (line_idx, line) in self.todo_text.lines().enumerate() {
            for (c_idx, c) in line.chars().enumerate() {
                let char_idx = self.todo_text.line_to_char(line_idx).unwrap() + c_idx; //TODO
                let cursor_status = self.cursors.get_cursor_status_for_char(char_idx);
                let pos = XY::new(c_idx as u16, line_idx as u16);

                // TODO optimise
                let text = format!("{}", c);
                let tr = if c == '\n' { NEWLINE } else { text.as_str() };

                match cursor_status {
                    CursorStatus::None => {
                        output.print_at(pos, theme.default_text(false), tr);
                    }
                    CursorStatus::WithinSelection => {
                        output.print_at(pos, theme.default_text(true), tr);
                    }
                    CursorStatus::UnderCursor => {
                        output.print_at(pos, theme.cursor(), tr);
                    }
                }
            }
        }

        let one_beyond_limit = self.todo_text.len_chars();
        let last_line = self.todo_text.char_to_line(one_beyond_limit).unwrap();//TODO
        let x_beyond_last = one_beyond_limit - self.todo_text.line_to_char(last_line).unwrap(); //TODO

        let one_beyond_last_pos = XY::new(x_beyond_last as u16, last_line as u16);
        match self.cursors.get_cursor_status_for_char(one_beyond_limit) {
            CursorStatus::None => {}
            CursorStatus::WithinSelection => {
                output.print_at(one_beyond_last_pos, theme.default_text(true), BEYOND);
            }
            CursorStatus::UnderCursor => {
                output.print_at(one_beyond_last_pos, theme.cursor(), BEYOND);
            }
        }
    }
}