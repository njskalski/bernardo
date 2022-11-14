use crate::config::theme::Theme;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::action_trigger::ActionTrigger;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{WID, Widget};

struct BigList<T: Widget> {
    items: Vec<T>,
    pos: usize,
}

impl<T: Widget> Widget for BigList<T> {
    fn id(&self) -> WID {
        todo!()
    }

    fn typename(&self) -> &'static str {
        todo!()
    }

    fn min_size(&self) -> XY {
        todo!()
    }

    fn update_and_layout(&mut self, sc: SizeConstraint) -> XY {
        todo!()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        todo!()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        todo!()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        todo!()
    }

    fn anchor(&self) -> XY {
        todo!()
    }
}