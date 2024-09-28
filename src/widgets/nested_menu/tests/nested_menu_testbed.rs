use std::fmt::Debug;

use crate::mocks::mock_output::MockOutput;
use crate::mocks::mock_providers_builder::MockProvidersBuilder;
use crate::mocks::mock_tree_item::MockTreeItem;
use crate::mocks::nested_menu_interpreter::NestedMenuInterpreter;
use crate::primitives::tree::tree_node::TreeNode;
use crate::primitives::xy::XY;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widgets::nested_menu::widget::NestedMenuWidget;
use crate::widgets::tests::generic_widget_testbed::GenericWidgetTestbed;

#[derive(Debug, Eq, PartialEq)]
pub enum NestedMenuTestMsg {
    Text(String),
}

impl AnyMsg for NestedMenuTestMsg {}

fn item_to_msg(item: &MockTreeItem) -> Option<Box<dyn AnyMsg>> {
    if item.is_leaf() {
        Some(NestedMenuTestMsg::Text(item.name.clone()).boxed())
    } else {
        assert!(false);
        None
    }
}

pub type NestedMenuTestbed = GenericWidgetTestbed<NestedMenuWidget<String, MockTreeItem>, ()>;

impl NestedMenuTestbed {
    pub fn new(mock_data_set: MockTreeItem) -> Self {
        let size = XY::new(30, 20);

        let build_result = MockProvidersBuilder::new().build();
        let (output, recv) = MockOutput::new(size, false, build_result.providers.theme().clone());

        NestedMenuTestbed {
            widget: NestedMenuWidget::new(build_result.providers.clone(), mock_data_set, size).with_mapper(item_to_msg),
            additional_data: (),
            size,
            last_frame: None,
            mock_navcomp_pilot: build_result.side_channels.navcomp_pilot,
            output,
            recv,
            last_msg: None,
            providers: build_result.providers,
        }
    }
    pub fn nested_menu(&self) -> Option<NestedMenuInterpreter> {
        self.last_frame.as_ref().map(|frame| frame.get_nested_menus().next()).flatten()
    }
}
