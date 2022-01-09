use log::warn;
use ropey::Rope;

use crate::{AnyMsg, InputEvent, Output, SizeConstraint, Theme, Widget};
use crate::primitives::cursor_set::CursorSet;
use crate::primitives::xy::XY;
use crate::text::buffer_state::BufferState;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::common_edit_msgs::{apply_cme, CommonEditMsg, key_to_edit_msg};
use crate::widgets::editor_view::msg::EditorViewMsg;

const MIN_EDITOR_SIZE: XY = XY::new(32, 10);

struct EditorView {
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
        todo!()
    }
}