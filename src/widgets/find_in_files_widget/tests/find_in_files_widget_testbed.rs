use crate::fs::path::SPath;
use crate::mocks::mock_output::MockOutput;
use crate::primitives::xy::XY;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::widget::Widget;
use crate::widgets::find_in_files_widget::find_in_files_widget::FindInFilesWidget;
use crate::widgets::find_in_files_widget::tests::find_in_files_widget_interpreter::FindInFilesWidgetInterpreter;
use crate::widgets::tests::generic_widget_testbed::GenericWidgetTestbed;
use crate::widgets::tests::generic_widget_testbed_builder::GenericWidgetTestbedBuilder;

pub struct AdditionalData {
    pub root: SPath,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Msg {
    Query(String, Option<String>),
    Cancel,
}

impl AnyMsg for Msg {}

pub type FindInFilesWidgetTestbed = GenericWidgetTestbed<FindInFilesWidget, AdditionalData>;

pub type FindInFilesWidgetTestbedBuilder = GenericWidgetTestbedBuilder<FindInFilesWidget, AdditionalData>;

impl FindInFilesWidgetTestbedBuilder {
    pub fn build(self) -> FindInFilesWidgetTestbed {
        let size = self.size.unwrap_or(XY::new(30, 20));

        let build_result = self.providers.build();
        let (output, recv) = MockOutput::new(size, false, build_result.providers.theme().clone());

        FindInFilesWidgetTestbed {
            widget: FindInFilesWidget::new(self.additional_data.root.clone())
                .with_on_cancel(Some(|_| Msg::Cancel.someboxed()))
                .with_on_hit(Some(|widget| Msg::Query(
                    widget.get_query(), widget.get_filter(),
                ).someboxed()))
            ,
            additional_data: self.additional_data,
            size,
            providers: build_result.providers,
            last_frame: None,
            mock_navcomp_pilot: Some(build_result.side_channels.navcomp_pilot),
            output,
            recv,
            last_msg: None,
        }
    }
}

impl FindInFilesWidgetTestbed {
    pub fn interpreter(&self) -> Option<FindInFilesWidgetInterpreter<'_>> {
        let frame = self.frame_op()?;
        let meta = frame.metadata.iter().find(|item| item.typename == FindInFilesWidget::static_typename())?;

        Some(FindInFilesWidgetInterpreter::new(meta, frame))
    }
}
