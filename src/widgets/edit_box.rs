use log::{debug, warn};
use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Keycode;
use crate::io::output::{Metadata, Output};
use crate::primitives::common_edit_msgs::{apply_cem, CommonEditMsg, key_to_edit_msg};
use crate::primitives::cursor_set::CursorSet;
use crate::primitives::helpers;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget, WidgetAction};

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
    pub const TYPENAME: &'static str = "edit_box";

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

    pub fn is_empty(&self) -> bool {
        self.text.len_bytes() == 0 //TODO
    }

    pub fn set_text<'a, T: Into<&'a str>>(&mut self, text: T) {
        self.text = Rope::from(text.into());
        self.cursor_set.move_home(&self.text, false);
    }

    pub fn set_cursor_end(&mut self) {
        self.cursor_set.move_end(&self.text, false);
    }

    pub fn clear(&mut self) {
        self.text = Rope::new();
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
        Self::TYPENAME
    }

    fn min_size(&self) -> XY {
        XY::new(MIN_WIDTH, 1)
    }

    fn update_and_layout(&mut self, sc: SizeConstraint) -> XY {
        debug_assert!(sc.bigger_equal_than(self.min_size()));

        let x = sc.visible_hint().size.x;
        self.display_state.width = x;

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
                        Some(cem) => match cem {
                            // the 4 cases below are designed to NOT consume the event in case it cannot be used.
                            CommonEditMsg::CursorUp { selecting: _ } |
                            CommonEditMsg::CursorDown { selecting: _ } => None,
                            CommonEditMsg::CursorLeft { selecting: _ } if self.cursor_set.as_single().map(|c| c.a == 0).unwrap_or(false) => None,
                            CommonEditMsg::CursorRight { selecting: _ } if self.cursor_set.as_single()
                                .map(|c| c.a > self.text.len_chars()).unwrap_or(false) => None,
                            _ => Some(Box::new(EditBoxWidgetMsg::CommonEditMsg(cem))),
                        }
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
                if apply_cem(cem.clone(), &mut self.cursor_set, &mut self.text, 1, None).1 {
                    self.event_changed()
                } else {
                    None
                }
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        output.emit_metadata(
            Metadata {
                id: self.id(),
                typename: self.typename().to_string(),
                rect: output.size_constraint().visible_hint().clone(),
                focused,
            }
        );

        let primary_style = theme.highlighted(focused);
        helpers::fill_output(primary_style.background, output);

        let mut x: usize = 0;
        for (char_idx, g) in self.text.to_string().graphemes(true).enumerate() {
            let style = match theme.cursor_background(self.cursor_set.get_cursor_status_for_char(char_idx)) {
                Some(bg) => {
                    primary_style.with_background(if focused { bg } else { bg.half() })
                }
                None => primary_style,
            };
            output.print_at(
                XY::new(x as u16, 0), //TODO
                style,
                g,
            );
            x += g.width();
        }
        // one character after
        {
            let style = match theme.cursor_background(self.cursor_set.get_cursor_status_for_char(self.text.len_chars())) {
                Some(bg) => {
                    primary_style.with_background(if focused { bg } else { bg.half() })
                }
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
        let text_width = self.text.to_string().width() as u16; //TODO
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
        XY::ZERO
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum EditBoxWidgetMsg {
    Hit,
    CommonEditMsg(CommonEditMsg),
}

impl AnyMsg for EditBoxWidgetMsg {}


impl Default for EditBoxWidget {
    fn default() -> Self {
        EditBoxWidget::new()
    }
}