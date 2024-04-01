use crossbeam_channel::Receiver;

use crate::experiments::screen_shot::screenshot;
use crate::experiments::screenspace::Screenspace;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::output::{FinalOutput, Output};
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::Widget;

pub struct GenericWidgetTestbed<W: Widget> {
    pub widget: W,
    pub size: XY,
    pub providers: Providers,
    pub last_frame: Option<MetaOutputFrame>,

    pub output: MockOutput,
    pub recv: Receiver<MetaOutputFrame>,
    pub last_msg: Option<Box<dyn AnyMsg>>,
}

impl<W: Widget> GenericWidgetTestbed<W> {
    pub fn next_frame(&mut self) {
        self.output.clear().unwrap();
        self.widget.prelayout();
        self.widget.layout(Screenspace::full_output(self.size));
        self.widget.render(&self.providers.theme(), true, &mut self.output);

        self.output.end_frame().unwrap();

        let frame = self.recv.recv().unwrap();
        self.last_frame = Some(frame);
    }

    pub fn frame_op(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    pub fn screenshot(&self) -> bool {
        self.frame_op().map(|frame| screenshot(&frame.buffer)).unwrap_or(false)
    }

    pub fn send_input(&mut self, input: InputEvent) {
        self.last_msg = self.widget.act_on(input).1;
        self.next_frame();
    }
}

impl<W: Widget> WithWaitFor for GenericWidgetTestbed<W> {
    fn is_frame_based_wait(&self) -> bool {
        false
    }

    fn last_frame(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    fn set_last_frame(&mut self, meta_output_frame: MetaOutputFrame) {
        self.last_frame = Some(meta_output_frame);
    }

    fn output_receiver(&self) -> &Receiver<MetaOutputFrame> {
        &self.recv
    }
}
