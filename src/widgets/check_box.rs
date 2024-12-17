use crate::widget::widget::{Widget, WID};

use super::text_widget::TextWidget;

pub struct CheckBoxWidget {
  id: WID,
  enabled:bool,
  label: TextWidget, 
}

impl CheckBoxWidget {
    pub const TYPENAME: &'static str = "check_box";
    pub const CHECK_SYMBOL_SIZE: u8 = 3;
}

impl Widget for CheckBoxWidget {
    fn id(&self) -> WID {
        self.id
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
        todo!()
    }

    fn layout(&mut self, screenspace: crate::experiments::screenspace::Screenspace) {
        todo!()
    }

    fn on_input(&self, input_event: crate::io::input_event::InputEvent) -> Option<Box<dyn crate::widget::any_msg::AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn crate::widget::any_msg::AnyMsg>) -> Option<Box<dyn crate::widget::any_msg::AnyMsg>> {
        todo!()
    }

    fn render(&self, theme: &crate::config::theme::Theme, focused: bool, output: &mut dyn crate::io::output::Output) {
        todo!()
    }
}