use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Key::Enter;
use crate::widget::widget::{Widget, MsgConstraints, BaseWidget, get_new_widget_id};
use crate::primitives::xy::XY;
use unicode_segmentation::UnicodeSegmentation;
use crate::io::output::Output;
use crate::io::style::{TextStyle_WhiteOnBlack, TextStyle_WhiteOnBlue, Effect};

pub struct ButtonWidget<ParentMsg: MsgConstraints> {
    id : usize,
    enabled: bool,
    text : String,
    on_hit: Option<fn(&Self) -> Option<ParentMsg>>,
}

impl<ParentMsg: MsgConstraints> BaseWidget for ButtonWidget<ParentMsg> {
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
}

impl<ParentMsg: MsgConstraints> ButtonWidget<ParentMsg> {
    pub fn new(text : String) -> Self {
        ButtonWidget {
            id: get_new_widget_id(),
            enabled: true,
            text,
            on_hit: None,
        }
    }

    pub fn with_on_hit(self, on_hit: fn(&Self) -> Option<ParentMsg>) -> Self {
        ButtonWidget {
            id: self.id,
            enabled: self.enabled,
            on_hit: Some(on_hit),
            text : self.text
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

impl <ParentMsg : MsgConstraints> Widget<ParentMsg> for ButtonWidget<ParentMsg> {
    type LocalMsg = ButtonWidgetMsg;

    fn update(&mut self, msg: ButtonWidgetMsg) -> Option<ParentMsg> {
        match msg {
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

    fn on_input(&self, input_event: InputEvent) -> Option<ButtonWidgetMsg> {
        debug_assert!(
            self.enabled,
            "ButtonWidget: received input to disabled component!"
        );

        match input_event {
            KeyInput(Enter) => Some(ButtonWidgetMsg::Hit),
            _ => None,
        }
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
