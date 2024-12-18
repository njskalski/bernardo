use crossterm::event::KeyCode;
use log::warn;

use crate::io::keys::Keycode;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::io::input_event::InputEvent;
use crate::io::input_event::InputEvent::KeyInput;
use super::text_widget::TextWidget;

pub struct CheckBoxWidget {
  wid: WID,
  enabled:bool,
  label: TextWidget, 
}

impl CheckBoxWidget {
    pub const TYPENAME: &'static str = "check_box";
    pub const CHECK_SYMBOL_ENABLED: &'static str = "[X]";
    pub const CHECK_SYMBOL_DISABLED: &'static str = "[ ]";
    pub const CHECK_SYMBOL_SIZE: u16 = 3;

    pub fn new(label: TextWidget) -> Self {
        Self {
            wid: get_new_widget_id(),
            enabled: false,
            label
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Widget for CheckBoxWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn static_typename() -> &'static str
    where
        Self: Sized {
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

    fn layout(&mut self, screenspace: crate::experiments::screenspace::Screenspace) {
    }

    fn on_input(&self, input_event: crate::io::input_event::InputEvent) -> Option<Box<dyn crate::widget::any_msg::AnyMsg>> {
        return match input_event {
            KeyInput(key_event) => match key_event.keycode {
                Keycode::Enter => Some(Box::new(CheckBoxMsg::Hit)),
                _ => None
            }
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn crate::widget::any_msg::AnyMsg>) -> Option<Box<dyn crate::widget::any_msg::AnyMsg>> {
        let our_msg = msg.as_msg::<CheckBoxMsg>();
        if our_msg.is_none() {
            warn!("expecetd CheckBoxMsg, got {:?}", msg);
            return None;
        }
        self.enabled = !self.enabled;
        None
        


    }

    fn render(&self, theme: &crate::config::theme::Theme, focused: bool, output: &mut dyn crate::io::output::Output) {
        let text = theme.default_text(focused);

        let mut line_idx = 0;
        let binding = self.label.get_text();
        let mut line_it = binding.lines();

        while let Some(line) = line_it.next() {
            output.print_at(XY::new(Self::CHECK_SYMBOL_SIZE, line_idx), text, line); //TODO leaks etc
            line_idx += 1;
        }
        let mut checked_symbol = Self::CHECK_SYMBOL_ENABLED;
        if !self.enabled {
            checked_symbol = Self::CHECK_SYMBOL_DISABLED;
        }
        output.print_at(XY::new(0, 0), text, &checked_symbol);
    }

}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CheckBoxMsg {
    Hit,
}

impl AnyMsg for CheckBoxMsg{}

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
        let input_event = InputEvent::KeyInput(key_event);
        if let Some(msg) = checkbox.on_input(input_event) {
            checkbox.update(msg);
        }

        assert_eq!(checkbox.is_enabled(), true);

        if let Some(msg) = checkbox.on_input(input_event) {
            checkbox.update(msg);
        }

        assert_eq!(checkbox.is_enabled(), false);
    }

    // #[test]
}