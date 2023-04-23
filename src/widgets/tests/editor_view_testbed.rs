use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};

use crate::config::config::{Config, ConfigRef};
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::mock_fs::MockFS;
use crate::io::input_event::InputEvent;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_navcomp_provider::MockNavCompProviderPilot;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;
use crate::widgets::editor_view::editor_view::EditorView;

pub struct EditorViewTestbed {
    pub editor_view: EditorView,
    pub input_sender: Sender<InputEvent>,
    pub output_receiver: Receiver<MetaOutputFrame>,
    pub config: ConfigRef,
    pub clipboard: ClipboardRef,
    pub theme: Theme,
    pub last_frame: Option<MetaOutputFrame>,
    pub mock_navcomp_pilot: MockNavCompProviderPilot,
}




