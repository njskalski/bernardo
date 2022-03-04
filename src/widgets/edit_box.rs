use log::{debug, warn};
use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::primitives::cursor_set::{CursorSet, CursorStatus};
use crate::primitives::helpers;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::theme::Theme;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget, WidgetAction};
use crate::widgets::common_edit_msgs::{apply_cme, CommonEditMsg, key_to_edit_msg};
use crate::widgets::editor_view::msg::EditorViewMsg;

const MIN_WIDTH: u16 = 12;
const MAX_WIDTH: u16 = 80; //completely arbitrary

//TODO filter out the newlines on paste


struct EditBoxDisplayState {
    width: u16,
}

pub struct EditBoxWidget {
    id: WID,
    enabled: bool,
    // hit is basically pressing enter.
    on_hit: Option<WidgetAction<EditBoxWidget>>,
    on_change: Option<WidgetAction<EditBoxWidget>>,
    // miss is trying to make illegal move. Like backspace on empty, left on leftmost etc.
    on_miss: Option<WidgetAction<EditBoxWidget>>,
    text: ropey::Rope,
    cursor_set: CursorSet,

    max_width_op: Option<u16>,

    //display state
    display_state: EditBoxDisplayState,
}

impl EditBoxWidget {
    pub fn new() -> Self {
        EditBoxWidget {
            id: get_new_widget_id(),
            cursor_set: CursorSet::single(),
            enabled: true,
            text: "".into(),
            on_hit: None,
            on_change: None,
            on_miss: None,
            max_width_op: None,
            display_state: EditBoxDisplayState {
                width: MIN_WIDTH
            },
        }
    }

    pub fn with_max_width(self, max_width: u16) -> Self {
        EditBoxWidget {
            max_width_op: Some(max_width),
            ..self
        }
    }

    pub fn with_on_hit(self, on_hit: WidgetAction<EditBoxWidget>) -> Self {
        EditBoxWidget {
            on_hit: Some(on_hit),
            ..self
        }
    }

    pub fn with_on_change(self, on_change: WidgetAction<EditBoxWidget>) -> Self {
        EditBoxWidget {
            on_change: Some(on_change),
            ..self
        }
    }

    pub fn with_on_miss(self, on_miss: WidgetAction<EditBoxWidget>) -> Self {
        EditBoxWidget {
            on_miss: Some(on_miss),
            ..self
        }
    }

    pub fn with_enabled(self, enabled: bool) -> Self {
        EditBoxWidget { enabled, ..self }
    }

    pub fn with_text<'a, T: Into<&'a str>>(self, text: T) -> Self {
        EditBoxWidget {
            text: ropey::Rope::from_str(text.into()),
            ..self
        }
    }

    pub fn get_text(&self) -> &ropey::Rope {
        &self.text
    }

    pub fn set_text<'a, T: Into<&'a str>>(&mut self, text: T) {
        self.text = Rope::from(text.into())
    }

    fn event_changed(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_change.is_some() {
            self.on_change.unwrap()(self)
        } else {
            None
        }
    }

    fn event_miss(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_miss.is_some() {
            self.on_miss.unwrap()(self)
        } else {
            None
        }
    }

    fn event_hit(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_hit.is_some() {
            self.on_hit.unwrap()(self)
        } else {
            None
        }
    }
}


impl Widget for EditBoxWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        "EditBox"
    }

    fn min_size(&self) -> XY {
        XY::new(MIN_WIDTH, 1)
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        debug_assert!(sc.bigger_equal_than(self.min_size()));

        let x = sc.x().unwrap_or(self.max_width_op.unwrap_or(MAX_WIDTH));

        XY::new(x, 1)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug_assert!(
            self.enabled,
            "EditBoxWidgetMsg: received input to disabled component!"
        );

        return match input_event {
            KeyInput(key_event) => {
                if key_event.keycode == Keycode::Enter {
                    Some(Box::new(EditBoxWidgetMsg::Hit))
                } else {
                    match key_to_edit_msg(key_event) {
                        Some(cem) => Some(Box::new(EditBoxWidgetMsg::CommonEditMsg(cem))),
                        None => None,
                    }
                }
            }
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<EditBoxWidgetMsg>();
        if our_msg.is_none() {
            warn!("expecetd EditBoxWidgetMsg, got {:?}", msg);
            return None;
        }
        debug!("EditBox got {:?}", msg);

        return match our_msg.unwrap() {
            EditBoxWidgetMsg::Hit => self.event_hit(),
            EditBoxWidgetMsg::CommonEditMsg(cem) => {
                if apply_cme(*cem, &mut self.cursor_set, &mut self.text, 1) {
                    self.event_changed()
                } else {
                    None
                }
            }
            _ => None,
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let primary_style = theme.highlighted(focused);
        helpers::fill_output(primary_style.background, output);

        let mut x: usize = 0;
        for (char_idx, g) in self.text.to_string().graphemes(true).enumerate() {
            let style = match theme.cursor_background(self.cursor_set.get_cursor_status_for_char(char_idx)) {
                Some(bg) => primary_style.with_background(bg),
                None => primary_style,
            };
            output.print_at(
                XY::new(x as u16, 0), //TODO
                style,
                g,
            );
            x += g.width_cjk();
        }
        // one character after
        {
            let style = match theme.cursor_background(self.cursor_set.get_cursor_status_for_char(self.text.len_chars())) {
                Some(bg) => primary_style.with_background(bg),
                None => primary_style,
            };
            output.print_at(
                XY::new(x as u16, 0),
                style,
                " ",
            );
        }

        // if cursor is after the text, we need to add an offset, so the background does not
        // overwrite cursor style.
        let cursor_offset: u16 = self.cursor_set.max_cursor_pos() as u16 + 1; //TODO
        let text_width = self.text.to_string().width_cjk() as u16; //TODO
        let end_of_text = cursor_offset.max(text_width);

        // background after the text
        if self.display_state.width > end_of_text {
            let background_length = self.display_state.width - end_of_text;
            for i in 0..background_length {
                let pos = XY::new(end_of_text + i as u16, 0);

                output.print_at(
                    pos,
                    primary_style,
                    " ",
                );
            }
        }
    }

    fn anchor(&self) -> XY {
        ZERO
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EditBoxWidgetMsg {
    Hit,
    CommonEditMsg(CommonEditMsg),
}

impl AnyMsg for EditBoxWidgetMsg {}
