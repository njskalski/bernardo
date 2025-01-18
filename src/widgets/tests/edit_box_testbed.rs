use std::sync::Arc;

use crate::config::config::Config;
use crate::mocks::editbox_interpreter::EditWidgetInterpreter;
use crate::mocks::mock_output::MockOutput;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::tests::generic_widget_testbed::GenericWidgetTestbed;
use crate::widgets::tests::generic_widget_testbed_builder::GenericWidgetTestbedBuilder;

pub type EditBoxTestbed = GenericWidgetTestbed<EditBoxWidget, ()>;
pub type EditBoxTestbedBuilder = GenericWidgetTestbedBuilder<EditBoxWidget, ()>;

impl EditBoxTestbedBuilder {
    pub fn build(self) -> EditBoxTestbed {
        let size = XY::new(100, 1);

        let build_result = self.providers.build();
        let (output, recv) = MockOutput::new(size, false, build_result.providers.theme().clone());

        EditBoxTestbed {
            widget: EditBoxWidget::new(Arc::new(Config::default())),
            additional_data: (),
            size,
            last_frame: None,
            mock_navcomp_pilot: Some(build_result.side_channels.navcomp_pilot),
            output,
            recv,
            providers: build_result.providers,
            last_msg: None,
        }
    }
}

impl EditBoxTestbed {
    pub fn interpreter(&self) -> EditWidgetInterpreter<'_> {
        let frame = self.frame_op().unwrap();
        let meta = frame
            .metadata
            .iter()
            .find(|item| item.typename == EditBoxWidget::static_typename())
            .unwrap();

        EditWidgetInterpreter::new(meta, frame)
    }
}
