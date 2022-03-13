use log::warn;
use unicode_width::UnicodeWidthStr;

use crate::experiments::deref_str::DerefStr;
use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::theme::Theme;
use crate::primitives::xy::{XY, ZERO};
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget, WidgetAction};

pub struct ButtonWidget {
    id: usize,
    enabled: bool,
    text: Box<dyn DerefStr>,
    on_hit: Option<WidgetAction<Self>>,
}

impl Widget for ButtonWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        "Button"
    }

    fn min_size(&self) -> XY {
        XY::new((self.text.as_ref_str().width_cjk() + 2) as u16, 1)
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        debug_assert!(sc.bigger_equal_than(self.min_size()), "min_size {}, got {}", self.min_size(), sc);
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        if !self.enabled {
            warn!("ButtonWidget: received input to disabled component!");
        }

        return match input_event {
            KeyInput(key_event) => match key_event.keycode {
                Keycode::Enter => Some(Box::new(ButtonWidgetMsg::Hit)),
                _ => None
            }
            _ => None
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<ButtonWidgetMsg>();
        if our_msg.is_none() {
            warn!("expecetd ButtonWidgetMsg, got {:?}", msg);
            return None;
        }

        match our_msg.unwrap() {
            ButtonWidgetMsg::Hit => {
                if self.on_hit.is_none() {
                    None
                } else {
                    self.on_hit.unwrap()(&self)
                }
            }
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let mut full_text = "[".to_string() + self.text.as_ref_str() + "]";

        let style = if focused {
            theme.highlighted(true)
        } else {
            theme.ui.non_focused
        };

        if focused {
            // style.effect = Effect::Underline;
            full_text = ">".to_string() + self.text.as_ref_str() + "<"
        }

        output.print_at((0, 0).into(), style, full_text.as_str());
    }

    fn anchor(&self) -> XY {
        ZERO
    }
}

impl ButtonWidget {
    pub fn new(text: Box<dyn DerefStr>) -> Self {
        ButtonWidget {
            id: get_new_widget_id(),
            enabled: true,
            text,
            on_hit: None,
        }
    }

    pub fn with_on_hit(self, on_hit: WidgetAction<Self>) -> Self {
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
            text: self.text,
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
