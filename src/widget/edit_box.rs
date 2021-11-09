use std::any::Any;
use std::borrow::Borrow;
use std::ops::Deref;

use log::warn;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Key;
use crate::io::keys::Key::Enter;
use crate::io::output::Output;
use crate::io::style::{
    TextStyle_WhiteOnBlack, TextStyle_WhiteOnBlue, TextStyle_WhiteOnBrightYellow,
    TextStyle_WhiteOnRedish, TextStyle_WhiteOnYellow,
};
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::edit_box::EditBoxWidgetMsg::Letter;
use crate::widget::widget::{get_new_widget_id, WID, Widget, WidgetAction};

pub struct EditBoxWidget {
    id: WID,
    enabled: bool,
    // hit is basically pressing enter.
    on_hit: Option<WidgetAction<EditBoxWidget>>,
    on_change: Option<WidgetAction<EditBoxWidget>>,
    // miss is trying to make illegal move. Like backspace on empty, left on leftmost etc.
    on_miss: Option<WidgetAction<EditBoxWidget>>,
    text: String,
    cursor: usize,
}

impl EditBoxWidget {
    pub fn new() -> Self {
        EditBoxWidget {
            id: get_new_widget_id(),
            cursor: 0,
            enabled: true,
            text: "".into(),
            on_hit: None,
            on_change: None,
            on_miss: None,
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

    pub fn with_text(self, text: String) -> Self {
        EditBoxWidget { text, ..self }
    }

    pub fn get_text(&self) -> &String {
        &self.text
    }

    fn event_changed(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_change.is_some() {
            self.on_change.unwrap()(self)
        } else {
            None
        }
    }

    fn event_miss(&self) -> Option<Box<AnyMsg>> {
        if self.on_miss.is_some() {
            self.on_miss.unwrap()(self)
        } else {
            None
        }
    }

    fn event_hit(&self) -> Option<Box<AnyMsg>> {
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

    fn min_size(&self) -> XY {
        XY::new(12, 1)
    }

    fn layout(&mut self, max_size: XY) -> XY {
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug_assert!(
            self.enabled,
            "EditBoxWidgetMsg: received input to disabled component!"
        );

        match input_event {
            KeyInput(Key::Enter) => Some(Box::new(EditBoxWidgetMsg::Hit)),
            KeyInput(Key::Letter(ch)) => Some(Box::new(EditBoxWidgetMsg::Letter(ch))),
            KeyInput(Key::Backspace) => Some(Box::new(EditBoxWidgetMsg::Backspace)),
            KeyInput(Key::ArrowLeft) => Some(Box::new(EditBoxWidgetMsg::ArrowLeft)),
            KeyInput(Key::ArrowRight) => Some(Box::new(EditBoxWidgetMsg::ArrowRight)),
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<EditBoxWidgetMsg>();
        if our_msg.is_none() {
            warn!("expecetd EditBoxWidgetMsg, got {:?}", msg);
            return None;
        }

        return match our_msg.unwrap() {
            EditBoxWidgetMsg::Hit => self.event_hit(),
            EditBoxWidgetMsg::Letter(ch) => {
                let mut new_text = self
                    .text
                    .graphemes(true)
                    .take(self.cursor)
                    .fold("".to_owned(), |a, b| a + b);

                new_text += ch.to_string().as_str(); //TODO: make this conversion better?

                new_text += self
                    .text
                    .graphemes(true)
                    .skip(self.cursor)
                    .fold("".to_owned(), |a, b| a + b)
                    .as_str();

                self.text = new_text;
                self.cursor += 1;

                self.event_changed()
            }
            EditBoxWidgetMsg::Backspace => {
                if self.cursor == 0 {
                    self.event_miss()
                } else {
                    self.cursor -= 1;
                    let mut new_text = self
                        .text
                        .graphemes(true)
                        .take(self.cursor)
                        .fold("".to_owned(), |a, b| a + b);
                    new_text += self
                        .text
                        .graphemes(true)
                        .skip(self.cursor + 1)
                        .fold("".to_owned(), |a, b| a + b)
                        .as_str();
                    self.text = new_text;
                    self.event_changed()
                }
            }
            EditBoxWidgetMsg::ArrowLeft => {
                if self.cursor == 0 {
                    self.event_miss()
                } else {
                    self.cursor -= 1;
                    None
                }
            }
            EditBoxWidgetMsg::ArrowRight => {
                if self.cursor >= self.text.len() {
                    self.event_miss()
                } else {
                    self.cursor += 1;
                    None
                }
            }
            _ => None,
        };
    }

    fn get_focused(&self) -> &dyn Widget {
        self
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        self
    }

    fn render(&self, focused: bool, output: &mut Output) {
        let mut primary_style = if self.enabled {
            if focused {
                TextStyle_WhiteOnBrightYellow
            } else {
                TextStyle_WhiteOnYellow
            }
        } else {
            TextStyle_WhiteOnBlack
        };

        let cursor_style = if self.enabled && focused {
            TextStyle_WhiteOnRedish
        } else {
            primary_style
        };

        let befor_cursor = self
            .text
            .graphemes(true)
            // .enumerate()
            .take(self.cursor)
            .map(|g| g.into())
            .fold("".to_string(), |a, b| a + b);

        let cursor_pos = self
            .text
            .graphemes(true)
            .take(self.cursor)
            .map(|g| g.width())
            .fold(0, |a, b| a + b);

        let at_cursor = self
            .text
            .graphemes(true)
            .skip(self.cursor)
            .next()
            .unwrap_or(" ");

        let after_cursor = self
            .text
            .graphemes(true)
            .skip(self.cursor + 1)
            .map(|g| g.into())
            .fold("".to_string(), |a, b| a + b);

        output.print_at((0, 0).into(), primary_style, befor_cursor.as_str());
        output.print_at((cursor_pos, 0).into(), cursor_style, at_cursor);
        if after_cursor.len() > 0 {
            output.print_at(
                (cursor_pos + 1, 0).into(),
                primary_style,
                after_cursor.as_str(),
            );
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EditBoxWidgetMsg {
    Hit,
    Letter(char),
    Backspace,
    ArrowLeft,
    ArrowRight,
}

impl AnyMsg for EditBoxWidgetMsg {}
