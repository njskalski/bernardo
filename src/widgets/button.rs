use log::warn;
use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::primitives::printable::Printable;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, Widget, WidgetAction, WID};

// TODO add fixed size and tests

pub struct ButtonWidget {
    id: usize,
    enabled: bool,
    text: Box<dyn Printable>,
    on_hit: Option<WidgetAction<Self>>,

    fill_x: bool,
    last_size_x: Option<u16>,
}

impl Widget for ButtonWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn full_size(&self) -> XY {
        XY::new((self.text.screen_width() + 2) as u16, 1)
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.last_size_x = Some(screenspace.output_size().x);
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        if !self.enabled {
            warn!("ButtonWidget: received input to disabled component!");
        }

        return match input_event {
            KeyInput(key_event) => match key_event.keycode {
                Keycode::Enter => Some(Box::new(ButtonWidgetMsg::Hit)),
                _ => None,
            },
            _ => None,
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
                    (self.on_hit.as_ref().unwrap())(&self)
                }
            }
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        {
            let size = XY::new(crate::unpack_unit!(self.last_size_x, "render before layout",), 1);

            output.emit_metadata(crate::io::output::Metadata {
                id: self.id(),
                typename: self.typename().to_string(),
                rect: crate::primitives::rect::Rect::from_zero(size),
                focused,
            });
        }

        let style = if focused { theme.highlighted(true) } else { theme.ui.non_focused };

        let mut xy = XY::ZERO;

        output.print_at(xy, style, if focused { ">" } else { "[" });
        xy += XY::new(1, 0);

        for grapheme in self.text.graphemes() {
            output.print_at(xy, style, grapheme);
            xy += XY::new(grapheme.width() as u16, 0);
        }

        output.print_at(xy, style, if focused { "<" } else { "]" });
    }

    fn kite(&self) -> XY {
        XY::ZERO
    }
}

impl ButtonWidget {
    pub const TYPENAME: &'static str = "button";

    pub fn new(text: Box<dyn Printable>) -> Self {
        ButtonWidget {
            id: get_new_widget_id(),
            enabled: true,
            text,
            on_hit: None,
            fill_x: false,
            last_size_x: None,
        }
    }

    pub fn with_on_hit(self, on_hit: WidgetAction<Self>) -> Self {
        ButtonWidget {
            on_hit: Some(on_hit),
            ..self
        }
    }

    pub fn with_enabled(self, enabled: bool) -> Self {
        ButtonWidget { enabled, ..self }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }

    pub fn set_fill_x(&mut self, new_fill_x: bool) {
        self.fill_x = new_fill_x;
    }

    pub fn with_fill_x(self) -> Self {
        Self { fill_x: true, ..self }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ButtonWidgetMsg {
    Hit,
    // Focus,
    // LostFocus
}

impl AnyMsg for ButtonWidgetMsg {}
