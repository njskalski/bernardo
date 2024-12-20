use log::warn;

use super::text_widget::TextWidget;
use crate::config::theme::Theme;
use crate::io::input_event::InputEvent::KeyInput;
use crate::io::keys::Keycode;
use crate::io::sub_output::SubOutput;
use crate::primitives::color::Color;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, Widget, WID};

pub struct CheckBoxWidget {
    wid: WID,
    enabled: bool,
    label: TextWidget,
    text_widget_theme: Theme, //ToDo Is it good design?
}

impl CheckBoxWidget {
    pub const TYPENAME: &'static str = "check_box";
    const CHECK_SYMBOL_ENABLED: &'static str = "[X]";
    const CHECK_SYMBOL_DISABLED: &'static str = "[ ]";
    const CHECK_SYMBOL_SIZE: u16 = 3;

    pub fn new(label: TextWidget) -> Self {
        let clicked_color = Color::new(255, 255, 153);
        let mut theme = Theme::default();
        theme.ui.focused.set_foreground(clicked_color);
        theme.ui.non_focused.set_foreground(clicked_color);
        
        Self {
            wid: get_new_widget_id(),
            enabled: false,
            label,
            text_widget_theme: theme,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle(&mut self) {
        self.enabled =!self.enabled;
    }
}

impl Widget for CheckBoxWidget {
    fn id(&self) -> WID {
        self.wid
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

    fn full_size(&self) -> crate::primitives::xy::XY {
        let mut size = self.label.full_size();
        size.x += Self::CHECK_SYMBOL_SIZE;
        size
    }

    fn layout(&mut self, screenspace: crate::experiments::screenspace::Screenspace) {}

    fn on_input(&self, input_event: crate::io::input_event::InputEvent) -> Option<Box<dyn crate::widget::any_msg::AnyMsg>> {
        return match input_event {
            KeyInput(key_event) => match key_event.keycode {
                Keycode::Enter => Some(Box::new(CheckBoxMsg::Hit)),
                _ => None,
            },
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn crate::widget::any_msg::AnyMsg>) -> Option<Box<dyn crate::widget::any_msg::AnyMsg>> {
        let our_msg = msg.as_msg::<CheckBoxMsg>();
        if our_msg.is_none() {
            warn!("expecetd CheckBoxMsg, got {:?}", msg);
            return None;
        }
        self.toggle();
        None
    }

    fn render(&self, theme: &crate::config::theme::Theme, focused: bool, output: &mut dyn crate::io::output::Output) {
        let text = theme.default_text(focused);
        let (checked_symbol, label_theme) = if self.enabled {
            (Self::CHECK_SYMBOL_ENABLED, &self.text_widget_theme)
        } else {
            (Self::CHECK_SYMBOL_DISABLED, theme)
        };
        
        output.print_at(XY::ZERO, text, &checked_symbol);
        let sub_output = &mut SubOutput::new(output, Rect::new(XY::new(Self::CHECK_SYMBOL_SIZE + 1, 0), self.label.text_size()));
        self.label.render(label_theme, focused, sub_output);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CheckBoxMsg {
    Hit,
}

impl AnyMsg for CheckBoxMsg {}

#[cfg(test)]
mod tests {
    use crate::io::keys::{Key, Modifiers};

    use super::*;

    #[test]
    fn test_check_box_click() {
        let label = TextWidget::new(Box::new("Test Checkbox".to_string()));
        let mut checkbox = CheckBoxWidget::new(label);
        assert_eq!(checkbox.is_enabled(), false);
        let key_event = Key {
            keycode: Keycode::Enter,
            modifiers: Modifiers::default(),
        };
        let input_event = KeyInput(key_event);
        if let Some(msg) = checkbox.on_input(input_event) {
            checkbox.update(msg);
        }

        assert_eq!(checkbox.is_enabled(), true);

        if let Some(msg) = checkbox.on_input(input_event) {
            checkbox.update(msg);
        }

        assert_eq!(checkbox.is_enabled(), false);
    }


}
