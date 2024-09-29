//TODO Task #56, Reduce ContextMenuTestbed to use as much code from GenericWidgetTestbed as possible.

use crate::mocks::context_menu_interpreter::ContextMenuInterpreter;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::mock_tree_item::MockTreeItem;
use crate::primitives::xy::XY;
use crate::widgets::context_menu::widget::ContextMenuWidget;
use crate::widgets::tests::generic_widget_testbed::GenericWidgetTestbed;
use crate::widgets::tests::generic_widget_testbed_builder::GenericWidgetTestbedBuilder;

pub struct AdditionalData {
    pub root: MockTreeItem,
}

pub type ContextMenuTestbed = GenericWidgetTestbed<ContextMenuWidget<String, MockTreeItem>, AdditionalData>;
pub type ContextMenuTestbedBuilder = GenericWidgetTestbedBuilder<ContextMenuWidget<String, MockTreeItem>, AdditionalData>;

impl ContextMenuTestbedBuilder {
    pub fn build(self) -> ContextMenuTestbed {
        let size = self.size.unwrap_or(XY::new(30, 20));
        let build_result = self.providers.build();

        let (output, recv) = MockOutput::new(size, false, build_result.providers.theme().clone());

        ContextMenuTestbed {
            widget: ContextMenuWidget::new(build_result.providers.clone(), self.additional_data.root.clone()),
            additional_data: self.additional_data,
            size,
            providers: build_result.providers,
            last_frame: None,
            mock_navcomp_pilot: None,
            output,
            recv,
            last_msg: None,
        }
    }
}

// pub struct ContextMenuTestbed {
//     context_menu: ContextMenuWidget<String, MockTreeItem>,
//     size: XY,
//     config: ConfigRef,
//     theme: Theme,
//     last_frame: Option<MetaOutputFrame>,
//     last_msg: Option<Box<dyn AnyMsg>>,
//
//     output: MockOutput,
//     recv: Receiver<MetaOutputFrame>,
// }

impl ContextMenuTestbed {
    // pub fn new(mock_data_set: MockTreeItem) -> Self {
    //     let size = XY::new(30, 20);
    //     let providers = MockProvidersBuilder::new().build().providers;
    //
    //     let theme: Theme = Default::default();
    //
    //     let (output, recv) = MockOutput::new(size, false, theme.clone());
    //
    //     ContextMenuTestbed {
    //         widget: ContextMenuWidget::new(providers, mock_data_set),
    //         additional_data: (),
    //         size,
    //         theme,
    //         last_frame: None,
    //         last_msg: None,
    //         output,
    //         recv,
    //         self.providers.unwrap(),
    //         mock_navcomp_pilot: providers.,
    //     }
    // }
    pub fn context_menu(&self) -> Option<ContextMenuInterpreter<'_>> {
        self.last_frame.as_ref().map(|frame| frame.get_context_menus().next()).flatten()
    }


    pub fn has_items<'a, I: Iterator<Item=&'a str>>(&self, items: I) -> bool {
        for item_label in items {
            if self
                .context_menu()
                .unwrap()
                .tree_view()
                .items()
                .iter()
                .find(|item| item.label.as_str() == item_label)
                .is_none()
            {
                return false;
            }
        }

        true
    }

    pub fn has_none_of_items<'a, I: Iterator<Item=&'a str>>(&self, items: I) -> bool {
        for item_label in items {
            if self
                .context_menu()
                .unwrap()
                .tree_view()
                .items()
                .iter()
                .find(|item| item.label.as_str() == item_label)
                .is_some()
            {
                return false;
            }
        }

        true
    }
}