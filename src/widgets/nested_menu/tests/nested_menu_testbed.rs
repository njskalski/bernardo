use std::fmt::Debug;

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::screen_shot::screenshot;
use crate::experiments::screenspace::Screenspace;
use crate::gladius::paradigm::recursive_treat_views;
use crate::io::input_event::InputEvent;
use crate::io::output::FinalOutput;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::mock_providers_builder::MockProvidersBuilder;
use crate::mocks::nested_menu_interpreter::NestedMenuInterpreter;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::tree::tree_node::TreeNode;
use crate::primitives::xy::XY;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::widget::Widget;
use crate::widgets::nested_menu::tests::mock_provider::MockNestedMenuItem;
use crate::widgets::nested_menu::widget::NestedMenuWidget;

#[derive(Debug, Eq, PartialEq)]
pub enum NestedMenuTestMsg {
    Text(String),
}

impl AnyMsg for NestedMenuTestMsg {}

fn item_to_msg(item: &MockNestedMenuItem) -> Option<Box<dyn AnyMsg>> {
    if item.is_leaf() {
        Some(NestedMenuTestMsg::Text(item.name.clone()).boxed())
    } else {
        assert!(false);
        None
    }
}

pub struct NestedMenuTestbed {
    pub nested_menu: NestedMenuWidget<String, MockNestedMenuItem>,
    pub size: XY,
    pub config: ConfigRef,
    pub theme: Theme,
    pub last_frame: Option<MetaOutputFrame>,
    pub last_msg: Option<Box<dyn AnyMsg>>,
}

impl NestedMenuTestbed {
    pub fn new(mock_data_set: MockNestedMenuItem) -> Self {
        let size = XY::new(30, 20);
        let providers = MockProvidersBuilder::new().build().providers;

        NestedMenuTestbed {
            nested_menu: NestedMenuWidget::new(providers, mock_data_set, size).with_mapper(item_to_msg),
            size,
            config: Default::default(),
            theme: Default::default(),
            last_frame: None,
            last_msg: None,
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

    pub fn screenshot(&self) -> bool {
        self.frame_op().map(|frame| screenshot(&frame.buffer)).unwrap_or(false)
    }

    pub fn push_input(&mut self, input: InputEvent) {
        let (_, last_msg) = recursive_treat_views(&mut self.nested_menu, input);
        self.last_msg = last_msg;
        self.next_frame();
    }
}
