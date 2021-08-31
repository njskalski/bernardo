use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Key::Enter;
use crate::widget::widget::{BaseWidget, get_new_widget_id, WidgetAction};
use crate::primitives::xy::XY;
use unicode_segmentation::UnicodeSegmentation;
use crate::io::output::Output;
use crate::io::style::{TextStyle_WhiteOnBlack, TextStyle_WhiteOnBlue, Effect};
use crate::primitives::sized_xy::SizedXY;
use crate::widget::any_msg::AnyMsg;
use std::borrow::Borrow;
use log::warn;

pub struct ButtonWidget {
    id : usize,
    enabled: bool,
    text : String,
    on_hit: Option<WidgetAction<ButtonWidget>>,
}

impl BaseWidget for ButtonWidget {
    fn id(&self) -> usize {
        self.id
    }

    fn min_size(&self) -> XY {
        // TODO: count grapheme width
        XY::new((self.text.len() + 2) as u16, 1)
    }

    fn size(&self, max_size: XY) -> XY {
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        if !self.enabled {
            warn!("ButtonWidget: received input to disabled component!");
        }

        match input_event {
            KeyInput(Enter) => Some(Box::new(ButtonWidgetMsg::Hit)),
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<ButtonWidgetMsg>();
        if our_msg.is_none() {
            warn!("expecetd ButtonWidgetMsg, got {:?}", msg);
            return None
        }

        match our_msg.unwrap() {
            ButtonWidgetMsg::Hit => {
                if self.on_hit.is_none() {
                    None
                } else {
                    self.on_hit.unwrap()(&self)
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

    fn render(&self, focused : bool, output: &mut Output) {
        let mut full_text = "[".to_string() + &self.text + "]";

        let mut style = if self.enabled {
            TextStyle_WhiteOnBlue
        } else {
            TextStyle_WhiteOnBlack
        };

        if focused {
            style.effect = Effect::Underline;
            full_text = ">".to_string() + &self.text + "<"
        }

        output.print_at((0,0).into(), style, full_text.as_str());
    }
}

impl ButtonWidget {
    pub fn new(text : String) -> Self {
        ButtonWidget {
            id: get_new_widget_id(),
            enabled: true,
            text,
            on_hit: None,
        }
    }

    pub fn with_on_hit(self, on_hit: WidgetAction<ButtonWidget>) -> Self {
        ButtonWidget {
            on_hit: Some(on_hit),
            ..self
        }
    }

    pub fn with_enabled(self, enabled: bool) -> Self {
        ButtonWidget {
            id: self.id,
            enabled,
            on_hit: self.on_hit,
            text : self.text
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ButtonWidgetMsg {
    Hit,
    // Focus,
    // LostFocus
}

impl AnyMsg for ButtonWidgetMsg {}
