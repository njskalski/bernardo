use crate::mocks::editbox_interpreter::EditWidgetInterpreter;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::mock_providers_builder::MockProvidersBuilder;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::tests::generic_widget_testbed::GenericWidgetTestbed;

pub type EditBoxTestbed = GenericWidgetTestbed<EditBoxWidget>;

impl EditBoxTestbed {
    pub fn new() -> Self {
        let size = XY::new(100, 1);
        let build_result = MockProvidersBuilder::new().build();
        let (output, recv) = MockOutput::new(size, false, build_result.providers.theme().clone());

        Self {
            widget: EditBoxWidget::new(),
            size,
            last_frame: None,
            mock_navcomp_pilot: build_result.side_channels.navcomp_pilot,
            output,
            recv,
            providers: build_result.providers,
            last_msg: None,
        }
    }
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
