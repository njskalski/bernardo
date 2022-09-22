use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::experiments::deref_str::DerefStr;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, WID, Widget};

pub struct TextWidget {
    wid: WID,
    text: Box<dyn DerefStr>,
}

impl TextWidget {
    pub fn new(text: Box<dyn DerefStr>) -> Self {
        Self {
            wid: get_new_widget_id(),
            text,
        }
    }

    pub fn text_size(&self) -> XY {
        let mut size = XY::ZERO;
        for (idx, line) in self.text.as_ref_str().lines().enumerate() {
            size.x = size.x.max(line.width() as u16); // TODO
            size.y = (idx + 1) as u16; //TODO
        }

        size
    }
}

impl Widget for TextWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "TextWidget"
    }

    fn min_size(&self) -> XY {
        self.text_size()
    }

    fn update_and_layout(&mut self, _sc: SizeConstraint) -> XY {
        self.text_size()
    }

    fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, _msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let text = theme.default_text(focused);

        for (idx, line) in self.text.as_ref_str().lines().enumerate() {
            output.print_at(XY::new(0, idx as u16), text, line); //TODO
        }
    }
}