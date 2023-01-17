use std::borrow::BorrowMut;
use std::fmt::{Debug, Formatter};

use bernardo::config::theme::Theme;
use bernardo::io::buffer_output::BufferOutput;
use bernardo::io::input_event::InputEvent;
use bernardo::io::keys::{Key, Keycode};
use bernardo::io::output::Output;
use bernardo::primitives::scroll::ScrollDirection;
use bernardo::primitives::size_constraint::SizeConstraint;
use bernardo::primitives::xy::XY;
use bernardo::widget::any_msg::AnyMsg;
use bernardo::widget::widget::{get_new_widget_id, WID, Widget};
use bernardo::widgets::dump_visualizer_widget::DumpVisualizerWidget;
use bernardo::widgets::with_scroll::WithScroll;

pub struct ReaderMainWidget {
    wid: WID,
    main_display: WithScroll<DumpVisualizerWidget>,
}

impl ReaderMainWidget {
    pub fn new(dump: BufferOutput) -> Self {
        Self {
            wid: get_new_widget_id(),
            main_display: WithScroll::new(
                ScrollDirection::Both,
                DumpVisualizerWidget::new()
                    .with_dump(dump)
                ,
            ),
        }
    }
}

#[derive(Clone, Debug)]
enum ReaderMainWidgetMsg {
    Close,
}

impl AnyMsg for ReaderMainWidgetMsg {}

impl Widget for ReaderMainWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "ReaderMainWidget"
    }

    fn size(&self) -> XY {
        XY::new(10, 10)
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        self.main_display.layout(sc)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        Some(&self.main_display)
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        Some(&mut self.main_display)
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        self.main_display.render(theme, focused, output)
    }
}