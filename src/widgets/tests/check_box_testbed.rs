use crate::mocks::editbox_interpreter::EditWidgetInterpreter;
use crate::mocks::mock_output::MockOutput;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;
use crate::widgets::check_box::CheckBoxWidget;
use crate::widgets::tests::generic_widget_testbed::GenericWidgetTestbed;
use crate::widgets::tests::generic_widget_testbed_builder::GenericWidgetTestbedBuilder;
use crate::widgets::text_widget::TextWidget;

pub type CheckBoxTestbed = GenericWidgetTestbed<CheckBoxWidget, ()>;
pub type CheckBoxTestbedBuilder = GenericWidgetTestbedBuilder<CheckBoxWidget, ()>;

impl CheckBoxTestbedBuilder {
  pub fn build(self, text: &'static str) -> CheckBoxTestbed {
      let size = XY::new(100, 1);

      let build_result = self.providers.build();
      let (output, recv) = MockOutput::new(size, false, build_result.providers.theme().clone());
      let label = TextWidget::new(Box::new(text));
      CheckBoxTestbed {
          widget: CheckBoxWidget::new(label),
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
