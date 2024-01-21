use streaming_iterator::StreamingIterator;
use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::printable::Printable;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};

pub struct TextWidget {
    wid: WID,
    text: Box<dyn Printable>,

    size_policy: SizePolicy,
}

impl TextWidget {
    pub const TYPENAME: &'static str = "text_widget";

    pub fn new(text: Box<dyn Printable>) -> Self {
        Self {
            wid: get_new_widget_id(),
            text,
            size_policy: SizePolicy::SELF_DETERMINED,
        }
    }

    pub fn with_size_policy(self, size_policy: SizePolicy) -> Self {
        Self { size_policy, ..self }
    }

    pub fn text_size(&self) -> XY {
        let mut size = XY::ZERO;

        let _debug_text = self.text.to_string();

        let mut line_it = self.text.lines();
        while let Some(line) = line_it.next() {
            size.x = size.x.max(line.width() as u16);
            size.y += 1;
        }

        size
    }

    pub fn get_text(&self) -> String {
        self.text.to_string()
    }

    pub fn set_text(&mut self, text: Box<dyn Printable>) {
        self.text = text;
    }
}

impl Widget for TextWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }
    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }
    fn size_policy(&self) -> SizePolicy {
        self.size_policy
    }

    fn full_size(&self) -> XY {
        self.text_size()
    }

    fn layout(&mut self, _screenspace: Screenspace) {}

    fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, _msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let text = theme.default_text(focused);

        let mut line_idx = 0;
        let mut line_it = self.text.lines();

        while let Some(line) = line_it.next() {
            output.print_at(XY::new(0, line_idx), text, line); //TODO leaks etc
            line_idx += 1;
        }
    }
}
