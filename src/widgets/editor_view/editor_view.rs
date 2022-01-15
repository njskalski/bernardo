use log::{error, warn};

use crate::{AnyMsg, InputEvent, Output, SizeConstraint, Theme, Widget};
use crate::primitives::arrow::Arrow;
use crate::primitives::cursor_set::{CursorSet, CursorStatus};
use crate::primitives::cursor_set_rect::cursor_set_to_rect;
use crate::primitives::xy::{XY, ZERO};
use crate::text::buffer::Buffer;
use crate::text::buffer_state::BufferState;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::common_edit_msgs::{apply_cme, cme_to_direction, CommonEditMsg, key_to_edit_msg};
use crate::widgets::editor_view::msg::EditorViewMsg;

const MIN_EDITOR_SIZE: XY = XY::new(32, 10);

const NEWLINE: &'static str = "⏎";
const BEYOND: &'static str = "⇱";

pub struct EditorView {
    wid: WID,
    cursors: CursorSet,

    last_size: Option<XY>,

    todo_text: BufferState,

    anchor: XY,
}

impl EditorView {
    pub fn new() -> EditorView {
        EditorView {
            wid: get_new_widget_id(),
            cursors: CursorSet::single(),
            last_size: None,
            todo_text: BufferState::new(),
            anchor: ZERO,
        }
    }


    pub fn with_buffer(self, buffer: BufferState) -> Self {
        EditorView {
            todo_text: buffer,
            ..self
        }
    }

    // This updates the "anchor" of view to match the direction of editing. Remember, the scroll will
    // follow the "anchor" with least possible change.
    fn update_anchor(&mut self, last_move_direction: Arrow) {
        // TODO test
        let cursor_rect = cursor_set_to_rect(&self.cursors, &self.todo_text);
        match last_move_direction {
            Arrow::Up => {
                if self.anchor.y > cursor_rect.upper_left().y {
                    self.anchor.y = cursor_rect.upper_left().y;
                }
            }
            Arrow::Down => {
                if self.anchor.y < cursor_rect.lower_right().y {
                    self.anchor.y = cursor_rect.lower_right().y;
                }
            }
            Arrow::Left => {
                if self.anchor.x > cursor_rect.upper_left().x {
                    self.anchor.x = cursor_rect.upper_left().x;
                }
            }
            Arrow::Right => {
                if self.anchor.x < cursor_rect.lower_right().x {
                    self.anchor.x = cursor_rect.lower_right().x;
                }
            }
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
        let size = sc.hint().size;
        self.last_size = Some(size);

        size
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
                    let page_height = match self.last_size {
                        Some(xy) => xy.y,
                        None => {
                            error!("received {:?} before retrieving last_size, using {} as page_height instead", cem, MIN_EDITOR_SIZE.y);
                            MIN_EDITOR_SIZE.y
                        }
                    };

                    // page_height as usize is safe, since page_height is u16 and usize is larger.
                    let _noop = apply_cme(*cem, &mut self.cursors, &mut self.todo_text, page_height as usize);

                    match cme_to_direction(*cem) {
                        None => {}
                        Some(direction) => self.update_anchor(direction)
                    };

                    None
                }
                _ => {
                    warn!("unhandled message {:?}", msg);
                    None
                }
            }
        };
    }

    fn render(&self, theme: &Theme, _focused: bool, output: &mut dyn Output) {
        for (line_idx, line) in self.todo_text.lines().enumerate()
            // skipping lines that cannot be visible, because they are before hint()
            .skip(output.size_constraint().hint().upper_left().y as usize) {
            // skipping lines that cannot be visible, because larger than the hint()
            if line_idx >= output.size_constraint().hint().lower_right().y as usize {
                break;
            }

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

    fn anchor(&self) -> XY {
        self.anchor
    }
}