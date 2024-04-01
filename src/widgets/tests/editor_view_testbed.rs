use crossbeam_channel::Receiver;

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::screen_shot::screenshot;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::output::{FinalOutput, Output};
use crate::mocks::editor_interpreter::EditorInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_navcomp_provider::MockNavCompProviderPilot;
use crate::mocks::mock_output::MockOutput;
use crate::mocks::with_wait_for::WithWaitFor;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;
use crate::widgets::editor_view::editor_view::EditorView;

pub struct EditorViewTestbed {
    pub editor_view: EditorView,
    pub size: XY,
    pub config: ConfigRef,
    pub clipboard: ClipboardRef,
    pub theme: Theme,
    pub last_frame: Option<MetaOutputFrame>,
    pub mock_navcomp_pilot: MockNavCompProviderPilot,

    pub output: MockOutput,
    pub recv: Receiver<MetaOutputFrame>,
}

impl EditorViewTestbed {
    pub fn editor(&self) -> Option<EditorInterpreter> {
        self.last_frame.as_ref().and_then(|frame| frame.get_editors().next())
    }

    pub fn next_frame(&mut self) {
        self.output.clear().unwrap();
        self.editor_view.prelayout();
        self.editor_view.layout(Screenspace::full_output(self.size));
        self.editor_view.render(&self.theme, true, &mut self.output);

        self.output.end_frame().unwrap();

        let frame = self.recv.recv().unwrap();
        self.last_frame = Some(frame);
    }

    pub fn frame_op(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    pub fn interpreter(&self) -> Option<EditorInterpreter<'_>> {
        self.frame_op()
            .and_then(|frame| EditorInterpreter::new(frame, frame.metadata.first().unwrap()))
    }

    pub fn screenshot(&self) -> bool {
        self.frame_op().map(|frame| screenshot(&frame.buffer)).unwrap_or(false)
    }

    pub fn push_input(&mut self, input: InputEvent) {
        self.editor_view.act_on(input);
        self.next_frame();
    }
}

impl WithWaitFor for EditorViewTestbed {
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
