use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::gladius::paradigm::recursive_treat_views;
use crate::io::input_event::InputEvent;
use crate::io::output::FinalOutput;
use crate::mocks::editbox_interpreter::EditWidgetInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_output::MockOutput;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;
use crate::widgets::edit_box::EditBoxWidget;

pub struct EditBoxTestbed {
    pub widget: EditBoxWidget,
    pub size: XY,
    pub theme: Theme,
    pub last_frame: Option<MetaOutputFrame>,
}

impl EditBoxTestbed {
    pub fn new() -> Self {
        Self {
            widget: EditBoxWidget::new(),
            size: XY::new(100, 1),
            theme: Default::default(),
            last_frame: None,
        }
    }

    pub fn next_frame(&mut self) {
        let (mut output, rcvr) = MockOutput::new(self.size, false, self.theme.clone());

        self.widget.prelayout();
        self.widget.layout(Screenspace::full_output(output.size()));
        self.widget.render(&self.theme, true, &mut output);

        output.end_frame().unwrap();

        let frame = rcvr.recv().unwrap();
        self.last_frame = Some(frame);
    }

    pub fn frame_op(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
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

    pub fn send_input(&mut self, input: InputEvent) {
        recursive_treat_views(&mut self.widget, input);
        self.next_frame();
    }
}
