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
    editor_view: EditorView,
    fsf: FsfRef,
    input_sender: Sender<InputEvent>,
    output_receiver: Receiver<MetaOutputFrame>,
    config: ConfigRef,
    clipboard: ClipboardRef,
    theme: Theme,
    last_frame: Option<MetaOutputFrame>,
    mock_navcomp_pilot: MockNavCompProviderPilot,
}




