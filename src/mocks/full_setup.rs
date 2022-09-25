use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender};
use which::Path;

use crate::config::config::{Config, ConfigRef};
use crate::config::theme::Theme;
use crate::experiments::clipboard::{Clipboard, ClipboardRef};
use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::mock_fs::MockFS;
use crate::gladius::run_gladius::run_gladius;
use crate::io::buffer_output::BufferOutput;
use crate::io::input_event::InputEvent;
use crate::mocks::mock_clipboard::MockClipboard;
use crate::mocks::mock_input::MockInput;
use crate::mocks::mock_output::MockOutput;
use crate::primitives::xy::XY;

const DEFAULT_MOCK_OUTPUT_SIZE: XY = XY::new(120, 36);

pub struct FullSetupBuilder {
    path: PathBuf,
    config: Option<Config>,
    files: Vec<PathBuf>,
    size: XY,
    recording: bool,
    step_frame: bool,
}

impl FullSetupBuilder {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FullSetupBuilder {
            path: path.as_ref().to_path_buf(),
            config: None,
            files: vec![],
            size: DEFAULT_MOCK_OUTPUT_SIZE,
            recording: false,
            step_frame: false,
        }
    }

    pub fn with_config(self, config: Config) -> Self {
        FullSetupBuilder {
            config: Some(config),
            ..self
        }
    }

    pub fn with_files<I: Iterator<Item=PathBuf>>(self, iter: I) -> Self {
        let files: Vec<PathBuf> = iter.collect();

        FullSetupBuilder {
            files,
            ..self
        }
    }

    pub fn with_recording(self) -> Self {
        Self {
            recording: true,
            ..self
        }
    }

    pub fn with_size(self, size: XY) -> Self {
        Self {
            size,
            ..self
        }
    }

    pub fn with_step_frame(self) -> Self {
        Self {
            step_frame: true,
            ..self
        }
    }
}

pub struct FullSetup {
    fsf: FsfRef,
    input_sender: Sender<InputEvent>,
    output_receiver: Receiver<BufferOutput>,
    config: ConfigRef,
    clipboard: ClipboardRef,
    theme: Theme,
    gladius_thread_handle: JoinHandle<()>,
}

impl FullSetupBuilder {
    pub fn build(self) -> FullSetup {
        // TODO setup logging too!

        let mock_fs = MockFS::generate_from_real(self.path).unwrap();
        let fsf = mock_fs.to_fsf();
        let (input, input_sender) = MockInput::new();
        let (output, output_receiver) = MockOutput::new(self.size, self.step_frame);
        let config: ConfigRef = Arc::new(self.config.unwrap_or(Config::default()));
        let clipboard: ClipboardRef = Arc::new(Box::new(MockClipboard::default()) as Box<dyn Clipboard + 'static>);

        let theme = Theme::default();

        let local_fsf = fsf.clone();
        let local_config = config.clone();
        let local_clipboard = clipboard.clone();
        let local_theme = theme.clone();
        let files = self.files;
        let recording = self.recording;

        let handle = std::thread::spawn(move || {
            run_gladius(local_fsf,
                        local_config,
                        local_clipboard,
                        input,
                        output,
                        files,
                        &local_theme,
                        recording)
        });

        FullSetup {
            fsf,
            input_sender,
            output_receiver,
            config,
            clipboard,
            theme,
            gladius_thread_handle: handle,
        }
    }
}

impl FullSetup {
    // TODO
    pub fn finish(self) -> FinishedFullRun {
        self.input_sender.send(InputEvent::KeyInput(self.config.keyboard_config.global.close)).unwrap();
        self.gladius_thread_handle.join().unwrap();

        FinishedFullRun {}
    }
}

pub struct FinishedFullRun {}