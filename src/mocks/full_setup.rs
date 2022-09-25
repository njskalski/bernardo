use std::ffi::OsStr;
use std::option::Option;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{Receiver, select, Sender};
use log::{error, LevelFilter};

use crate::config::config::{Config, ConfigRef};
use crate::config::theme::Theme;
use crate::experiments::clipboard::{Clipboard, ClipboardRef};
use crate::experiments::screen_shot::screenshot;
use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::mock_fs::MockFS;
use crate::gladius::run_gladius::run_gladius;
use crate::io::buffer_output::BufferOutput;
use crate::io::input_event::InputEvent;
use crate::mocks::mock_clipboard::MockClipboard;
use crate::mocks::mock_input::MockInput;
use crate::mocks::mock_navcomp_provider::MockNavCompProviderPilot;
use crate::mocks::mock_output::MockOutput;
use crate::primitives::xy::XY;
use crate::widgets::no_editor::NoEditorWidget;

pub struct FullSetupBuilder {
    path: PathBuf,
    config: Option<Config>,
    files: Vec<PathBuf>,
    size: XY,
    recording: bool,
    step_frame: bool,
    nav_comp_pilot: MockNavCompProviderPilot,
}

impl FullSetupBuilder {
    const DEFAULT_MOCK_OUTPUT_SIZE: XY = XY::new(120, 36);

    pub fn new<P: AsRef<OsStr>>(path: P) -> Self {
        FullSetupBuilder {
            path: PathBuf::from(path.as_ref()),
            config: None,
            files: vec![],
            size: Self::DEFAULT_MOCK_OUTPUT_SIZE,
            recording: false,
            step_frame: false,
            nav_comp_pilot: MockNavCompProviderPilot::new(),
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
    nav_comp_pilot: MockNavCompProviderPilot,
}

impl FullSetupBuilder {
    pub fn build(self) -> FullSetup {
        let mut logger_builder = env_logger::builder();
        logger_builder.filter_level(LevelFilter::Debug);
        logger_builder.init();

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
            nav_comp_pilot: MockNavCompProviderPilot::new(),
        }
    }
}

impl FullSetup {
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(1);

    pub fn wait_frame(&mut self) -> bool {
        select! {
            recv(self.output_receiver) -> frame_res => {
                match frame_res {
                    Ok(frame) => {
                        self.last_frame = Some(frame);
                        true
                    }
                    Err(e) => {
                        error!("failed retrieving frame : {:?}", e);
                        false
                    }
                }
            },
            default(Self::DEFAULT_TIMEOUT) => {
                error!("timeout");
                false
            },
        }
    }

    pub fn navcomp_pilot(&self) -> &MockNavCompProviderPilot {
        &self.nav_comp_pilot
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

    pub fn fsf(&self) -> &FsfRef {
        &self.fsf
    }

    /*
    returns lines which have cursors (editor must be opened)
     */
    // pub fn get_cursor_lines(&self) {}

    pub fn finish(self) -> FinishedFullSetupRun {
        self.input_sender.send(InputEvent::KeyInput(self.config.keyboard_config.global.close)).unwrap();
        self.gladius_thread_handle.join().unwrap();

        FinishedFullSetupRun {
            fsf: self.fsf,
            last_frame: self.last_frame,
            clipboard: self.clipboard,
        }
    }
}

pub struct FinishedFullSetupRun {
    pub fsf: FsfRef,
    pub last_frame: Option<BufferOutput>,
    pub clipboard: ClipboardRef,
}

impl FinishedFullSetupRun {
    pub fn screenshot(&self) -> bool {
        self.last_frame.as_ref().map(|frame| screenshot(frame)).unwrap_or(false)
    }
}