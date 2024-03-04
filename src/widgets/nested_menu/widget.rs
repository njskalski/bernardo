use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{WID, Widget};

/*
This describes a simple context menu.
For first version, options remain fixed (no adding/deleting)
 */

pub struct NestedMenuWidget {
    wid : WID,
}

impl NestedMenuWidget {

}

impl Widget for NestedMenuWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn static_typename() -> &'static str where Self: Sized {
        "NestedMenuWidget"
    }

    fn typename(&self) -> &'static str {
        "NestedMenuWidget"
    }

    fn full_size(&self) -> XY {
        todo!()
    }

    fn layout(&mut self, screenspace: Screenspace) {
        todo!()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        todo!()
    }
}