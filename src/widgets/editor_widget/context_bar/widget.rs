use crate::config::theme::Theme;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{WID, Widget};
use crate::widgets::editor_widget::context_bar::context_bar_item::ContextBarItem;
use crate::widgets::list_widget::list_widget::ListWidget;

pub struct ContextBarWidget {
    id: WID,
    list: ListWidget<ContextBarItem>,
}

impl ContextBarWidget {
    pub const TYPENAME: &'static str = "context_bar";
}

impl Widget for ContextBarWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
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

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        todo!()
    }
}