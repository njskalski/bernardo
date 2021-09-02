use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Key::Enter;
use crate::widget::edit_box::EditBoxWidgetMsg::Letter;
use crate::widget::widget::{get_new_widget_id, BaseWidget, WidgetAction};
use unicode_segmentation::UnicodeSegmentation;
use crate::io::keys::Key;
use crate::primitives::xy::XY;
use crate::io::output::Output;
use crate::io::style::{TextStyle_WhiteOnBlue, TextStyle_WhiteOnBlack, TextStyle_WhiteOnYellow, TextStyle_WhiteOnBrightYellow, TextStyle_WhiteOnRedish};
use unicode_width::UnicodeWidthStr;
use crate::widget::any_msg::AnyMsg;
use std::any::Any;
use std::borrow::Borrow;
use std::ops::Deref;
use log::warn;

pub struct EditBoxWidget {
    id: usize,
    enabled: bool,
    on_hit: Option<WidgetAction<EditBoxWidget>>,
    on_change: Option<WidgetAction<EditBoxWidget>>,
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
            on_change: None
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

    pub fn with_enabled(self, enabled: bool) -> Self {
        EditBoxWidget {
            enabled,
            ..self
        }
    }

    pub fn with_text(self, text : String) -> Self {
        EditBoxWidget {
            text,
            ..self
        }
    }

    pub fn get_text(&self) -> &String {
        &self.text
    }
}

impl BaseWidget for EditBoxWidget {
    fn id(&self) -> usize {
        self.id
    }

    fn min_size(&self) -> XY {
        XY::new(12, 1)
    }

    fn size(&self, max_size: XY) -> XY {
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug_assert!(
            self.enabled,
            "EditBoxWidgetMsg: received input to disabled component!"
        );

        match input_event {
            KeyInput(Enter) => Some(Box::new(EditBoxWidgetMsg::Hit)),
            KeyInput(Key::Letter(ch)) => Some(Box::new(EditBoxWidgetMsg::Letter(ch))),
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<EditBoxWidgetMsg>();
        if our_msg.is_none() {
            warn!("expecetd EditBoxWidgetMsg, got {:?}", msg);
            return None
        }

        match our_msg.unwrap() {
            EditBoxWidgetMsg::Hit => {
                if self.on_hit.is_none() {
                    None
                } else {
                    self.on_hit.unwrap()(&self)
                }
            }
            EditBoxWidgetMsg::Letter(ch) => {
                let mut iter = self.text.graphemes(true).into_iter();
                let mut new_text = self.text.graphemes(true)
                    .take(self.cursor)
                    .fold("".to_owned(), |a, b| a + b);

                new_text += ch.to_string().as_str(); //TODO: make this conversion better?

                new_text += self.text.graphemes(true).skip(self.cursor)
                    .fold("".to_owned(), |a, b| a + b).as_str();

                self.text = new_text.to_owned();
                self.cursor += 1;

                if self.on_change.is_some() {
                    self.on_change.unwrap()(self)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn get_focused(&self) -> &dyn BaseWidget {
        self
    }

    fn get_focused_mut(&mut self) -> &mut dyn BaseWidget {
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

        let befor_cursor = self.text.graphemes(true)
            // .enumerate()
            .take(self.cursor)
            .map(|g| g.into())
            .fold("".to_string(), |a, b| a + b);

        let cursor_pos = self.text.graphemes(true)
            .take(self.cursor)
            .map(|g| g.width())
            .fold(0, |a, b| a + b);

        let at_cursor = self.text.graphemes(true)
            .skip(self.cursor)
            .next()
            .unwrap_or(" ");

        let after_cursor = self.text.graphemes(true)
            .skip(self.cursor + 1)
            .map(|g| g.into())
            .fold("".to_string(), |a, b| a + b);

        output.print_at((0, 0).into(), primary_style, befor_cursor.as_str());
        output.print_at((cursor_pos, 0).into(), cursor_style, at_cursor);
        if after_cursor.len() > 0 {
            output.print_at((cursor_pos + 1, 0).into(), primary_style, after_cursor.as_str());
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EditBoxWidgetMsg {
    Hit,
    Letter(char),
}

impl AnyMsg for EditBoxWidgetMsg {}