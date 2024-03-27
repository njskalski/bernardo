use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::screen_shot::screenshot;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::output::FinalOutput;
use crate::mocks::context_menu_interpreter::ContextMenuInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::mock_providers_builder::MockProvidersBuilder;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::Widget;
use crate::widgets::context_menu::tests::mock_provider::MockNestedMenuItem;
use crate::widgets::context_menu::widget::ContextMenuWidget;

#[derive(Debug, Eq, PartialEq)]
pub enum ContextMenuTestMsg {
    Text(String),
}

impl AnyMsg for ContextMenuTestMsg {}

pub struct ContextMenuTestbed {
    pub context_menu: ContextMenuWidget<String, MockNestedMenuItem>,
    pub size: XY,
    pub config: ConfigRef,
    pub theme: Theme,
    pub last_frame: Option<MetaOutputFrame>,
    pub last_msg: Option<Box<dyn AnyMsg>>,
}

impl ContextMenuTestbed {
    pub fn new(mock_data_set: MockNestedMenuItem) -> Self {
        let size = XY::new(30, 20);
        let providers = MockProvidersBuilder::new().build().providers;

        ContextMenuTestbed {
            context_menu: ContextMenuWidget::new(providers, mock_data_set),
            size,
            config: Default::default(),
            theme: Default::default(),
            last_frame: None,
            last_msg: None,
        }
    }
    pub fn context_menu(&self) -> Option<ContextMenuInterpreter<'_>> {
        self.last_frame.as_ref().map(|frame| frame.get_context_menus().next()).flatten()
    }

    pub fn next_frame(&mut self) {
        let (mut output, rcvr) = MockOutput::new(self.size, false, self.theme.clone());

        self.context_menu.prelayout();
        self.context_menu.layout(Screenspace::full_output(output.size()));
        self.context_menu.render(&self.theme, true, &mut output);

        output.end_frame().unwrap();

        let frame = rcvr.recv().unwrap();
        self.last_frame = Some(frame);
    }

    pub fn frame_op(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    pub fn screenshot(&self) -> bool {
        self.frame_op().map(|frame| screenshot(&frame.buffer)).unwrap_or(false)
    }

    pub fn push_input(&mut self, input: InputEvent) {
        let (_, last_msg) = self.context_menu.act_on(input);
        self.last_msg = last_msg;
        self.next_frame();
    }
}
