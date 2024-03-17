use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::screen_shot::screenshot;
use crate::experiments::screenspace::Screenspace;
use crate::gladius::paradigm::recursive_treat_views;
use crate::io::input_event::InputEvent;
use crate::io::output::FinalOutput;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::nested_menu_interpreter::NestedMenuInterpreter;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;
use crate::widgets::nested_menu::tests::mock_provider::{get_mock_data, MockNestedMenuItem};
use crate::widgets::nested_menu::widget::NestedMenuWidget;

pub struct NestedMenuTestbed {
    pub nested_menu: NestedMenuWidget<String, MockNestedMenuItem>,
    pub size: XY,
    pub config: ConfigRef,
    pub theme: Theme,
    pub last_frame: Option<MetaOutputFrame>,
}

impl NestedMenuTestbed {
    pub fn new() -> Self {
        let size = XY::new(30, 20);

        NestedMenuTestbed {
            nested_menu: NestedMenuWidget::new(get_mock_data(), size),
            size,
            config: Default::default(),
            theme: Default::default(),
            last_frame: None,
        }
    }
    pub fn nested_menu(&self) -> Option<NestedMenuInterpreter> {
        self.last_frame.as_ref().map(|frame| frame.get_nested_menus().next()).flatten()
    }

    pub fn next_frame(&mut self) {
        let (mut output, rcvr) = MockOutput::new(self.size, false, self.theme.clone());

        self.nested_menu.prelayout();
        self.nested_menu.layout(Screenspace::full_output(output.size()));
        self.nested_menu.render(&self.theme, true, &mut output);

        output.end_frame().unwrap();

        let frame = rcvr.recv().unwrap();
        self.last_frame = Some(frame);
    }

    pub fn frame_op(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    // pub fn interpreter(&self) -> Option<EditorInterpreter<'_>> {
    //     self.frame_op()
    //         .and_then(|frame| EditorInterpreter::new(frame, frame.metadata.first().unwrap()))
    // }

    pub fn screenshot(&self) -> bool {
        self.frame_op().map(|frame| screenshot(&frame.buffer)).unwrap_or(false)
    }

    pub fn push_input(&mut self, input: InputEvent) {
        recursive_treat_views(&mut self.nested_menu, input);
        self.next_frame();
    }
}
