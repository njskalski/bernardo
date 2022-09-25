use std::ffi::OsStr;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender};
use log::error;
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
use crate::widgets::no_editor::NoEditorWidget;

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
    pub fn new<P: AsRef<OsStr>>(path: P) -> Self {
        FullSetupBuilder {
            path: PathBuf::from(path.as_ref()),
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

    pub fn with_files<P: AsRef<OsStr>, I: IntoIterator<Item=P>>(self, items: I) -> Self {
        let files: Vec<PathBuf> = items.into_iter().map(|p| PathBuf::from(p.as_ref())).collect();

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
    last_frame: Option<BufferOutput>,
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
            last_frame: None,
        }
    }
}

impl FullSetup {
    // TODO not finished
    pub fn finish(self) -> FinishedFullRun {
        self.input_sender.send(InputEvent::KeyInput(self.config.keyboard_config.global.close)).unwrap();
        self.gladius_thread_handle.join().unwrap();

        FinishedFullRun {}
    }

    pub fn wait_frame(&mut self) -> bool {
        match self.output_receiver.recv() {
            Ok(buffer) => {
                self.last_frame = Some(buffer);
                true
            }
            Err(e) => {
                error!("failed retrieving frame: {:?}", e);
                false
            }
        }
    }

    pub fn dry(&mut self) -> bool {
        let mut res = false;
        let mut iter = self.output_receiver.try_iter();
        while let Some(frame) = iter.next() {
            self.last_frame = Some(frame);
            res = true;
        }

        res
    }

    pub fn get_frame(&self) -> Option<&BufferOutput> {
        self.last_frame.as_ref()
    }

    /*
    Looks for default "no editor opened" text of NoEditorWidget.
     */
    pub fn is_editor_opened(&self) -> bool {
        if self.last_frame.as_ref().unwrap().to_string().find(NoEditorWidget::NO_EDIT_TEXT).is_some() {
            false
        } else {
            true
        }
    }

    /*
    returns lines which have cursors (editor must be opened)
     */
    pub fn get_cursor_lines(&self) {}
}

pub struct FinishedFullRun {}