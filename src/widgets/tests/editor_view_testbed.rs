use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};

use crate::config::config::{Config, ConfigRef};
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::screen_shot::screenshot;
use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::mock_fs::MockFS;
use crate::gladius::paradigm::recursive_treat_views;
use crate::io::input_event::InputEvent;
use crate::io::output::{FinalOutput, Metadata};
use crate::mocks::editor_interpreter::EditorInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_navcomp_provider::MockNavCompProviderPilot;
use crate::mocks::mock_output::MockOutput;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
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
}


impl EditorViewTestbed {
    pub fn editor(&self) -> Option<EditorInterpreter> {
        self.last_frame.as_ref().map(
            |frame| frame.get_editors().next()
        ).flatten()
    }

    pub fn next_frame(&mut self) {
        let (mut output, rcvr) = MockOutput::new(self.size, false, self.theme.clone());

        self.editor_view.prelayout();
        self.editor_view.layout(output.size(), Rect::from_zero(output.size()));
        self.editor_view.render(&self.theme, true, &mut output);

        output.end_frame().unwrap();

        let frame = rcvr.recv().unwrap();
        self.last_frame = Some(frame);
    }

    pub fn frame_op(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    pub fn interpreter(&self) -> Option<EditorInterpreter<'_>> {
        self.frame_op().map(|frame| {
            EditorInterpreter::new(frame, frame.metadata.first().unwrap())
        }).flatten()
    }

    pub fn screenshot(&self) -> bool {
        self.frame_op().map(|frame|
            screenshot(&frame.buffer)
        ).unwrap_or(false)
    }

    pub fn push_input(&mut self, input: InputEvent) {
        recursive_treat_views(&mut self.editor_view, input);
        self.next_frame();
    }
}
