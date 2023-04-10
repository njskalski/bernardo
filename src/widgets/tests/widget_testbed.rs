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

impl WidgetTestbedBuilder {
    pub const DEFAULT_MOCK_OUTPUT_SIZE: XY = XY::new(120, 36);

    pub fn with_config(self, config: Config) -> Self {
        Self {
            config: Some(Arc::new(config)),
            ..self
        }
    }

    pub fn with_size(self, size: XY) -> Self {
        Self {
            size,
            ..self
        }
    }

    pub fn build(self) -> WidgetTestbed {
        let fsf = self.fsf.unwrap_or(
            Box::new(MockFS::new("/testbed"))
        ).to_fsf();

        let (input, input_sender) = MockInput::new();
        let (output, output_receiver) = MockOutput::new(self.size, self.step_frame, theme.clone());


        WidgetTestbed {
            fsf,
            input_sender: (),
            output_receiver: (),
            config: Arc::new(Default::default()),
            clipboard: Arc::new(Box::new(())),
            theme: Default::default(),
            last_frame: None,
            mock_navcomp_pilot: (),
        }
    }
}

pub struct WidgetTestbedBuilder {
    size: XY,
    fsf: Option<Box<dyn FilesystemFront>>,
    input_sender: Sender<InputEvent>,
    output_receiver: Receiver<MetaOutputFrame>,
    config: Option<Config>,
    clipboard: ClipboardRef,
    theme: Theme,
    last_frame: Option<MetaOutputFrame>,
    mock_navcomp_pilot: MockNavCompProviderPilot,
}

pub struct WidgetTestbed {
    fsf: FsfRef,
    input_sender: Sender<InputEvent>,
    output_receiver: Receiver<MetaOutputFrame>,
    config: ConfigRef,
    clipboard: ClipboardRef,
    theme: Theme,
    last_frame: Option<MetaOutputFrame>,
    mock_navcomp_pilot: MockNavCompProviderPilot,
}




