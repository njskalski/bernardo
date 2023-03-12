use std::ffi::OsStr;
use std::option::Option;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{Receiver, select, Sender};
use log::{debug, error, LevelFilter, warn};

use crate::config::config::{Config, ConfigRef};
use crate::config::theme::Theme;
use crate::experiments::clipboard::{Clipboard, ClipboardRef};
use crate::experiments::screen_shot::screenshot;
use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::mock_fs::MockFS;
use crate::gladius::logger_setup::logger_setup;
use crate::gladius::navcomp_loader::NavCompLoader;
use crate::gladius::providers::Providers;
use crate::gladius::run_gladius::run_gladius;
use crate::io::input_event::InputEvent;
use crate::io::keys::{Key, Keycode};
use crate::mocks::code_results_interpreter::CodeResultsViewInterpreter;
use crate::mocks::editor_interpreter::EditorInterpreter;
use crate::mocks::fuzzy_search_interpreter::FuzzySearchInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_clipboard::MockClipboard;
use crate::mocks::mock_input::MockInput;
use crate::mocks::mock_navcomp_loader::MockNavcompLoader;
use crate::mocks::mock_navcomp_provider::{MockCompletionMatcher, MockNavCompEvent, MockNavCompProviderPilot, MockSymbolMatcher};
use crate::mocks::mock_output::MockOutput;
use crate::mocks::treeview_interpreter::TreeViewInterpreter;
use crate::primitives::xy::XY;
use crate::tsw::language_set::LanguageSet;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::widgets::code_results_view::code_results_widget::CodeResultsView;
use crate::widgets::tree_view;

pub struct FullSetupBuilder {
    path: PathBuf,
    config: Option<Config>,
    files: Vec<PathBuf>,
    size: XY,
    recording: bool,
    step_frame: bool,
    frame_based_wait: bool,
}

impl FullSetupBuilder {
    pub const DEFAULT_MOCK_OUTPUT_SIZE: XY = XY::new(120, 36);

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

    // Turn this on if you are debugging, and you don't want the default timeout to kick in.
    pub fn with_frame_based_wait(self) -> Self {
        FullSetupBuilder {
            frame_based_wait: true,
            ..self
        }
    }
}

pub struct FullSetup {
    fsf: FsfRef,
    input_sender: Sender<InputEvent>,
    output_receiver: Receiver<MetaOutputFrame>,
    config: ConfigRef,
    clipboard: ClipboardRef,
    theme: Theme,
    gladius_thread_handle: JoinHandle<()>,
    last_frame: Option<MetaOutputFrame>,
    frame_based_wait: bool,
    mock_navcomp_pilot: MockNavCompProviderPilot,
}

impl FullSetupBuilder {
    pub fn build(self) -> FullSetup {
        logger_setup(LevelFilter::Debug);

        let theme = Theme::default();

        let mock_fs = MockFS::generate_from_real(self.path).unwrap();
        let fsf = mock_fs.to_fsf();
        let (input, input_sender) = MockInput::new();
        let (output, output_receiver) = MockOutput::new(self.size, self.step_frame, theme.clone());
        let config: ConfigRef = Arc::new(self.config.unwrap_or(Config::default()));
        let clipboard: ClipboardRef = Arc::new(Box::new(MockClipboard::default()) as Box<dyn Clipboard + 'static>);

        let local_fsf = fsf.clone();
        let local_config = config.clone();
        let local_clipboard = clipboard.clone();
        let local_theme = theme.clone();
        let files = self.files;

        let tree_sitter = Arc::new(TreeSitterWrapper::new(LanguageSet::full()));

        let (mock_navcomp_event_sender, mock_navcomp_event_recvr) = crossbeam_channel::unbounded::<MockNavCompEvent>();
        let comp_matcher: Arc<RwLock<Vec<MockCompletionMatcher>>> = Arc::new(RwLock::new(Vec::new()));
        let symbol_matcher: Arc<RwLock<Vec<MockSymbolMatcher>>> = Arc::new(RwLock::new(Vec::new()));

        let mock_navcomp_pilot = MockNavCompProviderPilot::new(
            mock_navcomp_event_recvr,
            comp_matcher.clone(),
            symbol_matcher.clone(),
        );

        let mock_navcomp_loader = Arc::new(Box::new(
            MockNavcompLoader::new(
                mock_navcomp_event_sender,
                comp_matcher,
                symbol_matcher,
            )
        ) as Box<dyn NavCompLoader>
        );

        let providers = Providers::new(
            local_config,
            local_fsf,
            local_clipboard,
            local_theme,
            tree_sitter,
            mock_navcomp_loader,
        );

        let providers_clone = providers.clone();

        let handle = std::thread::spawn(move || {
            run_gladius(providers_clone,
                        input,
                        output,
                        files,
            )
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
            frame_based_wait: self.frame_based_wait,
            mock_navcomp_pilot,
        }
    }
}

impl FullSetup {
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(3);
    const DEFAULT_TIMEOUT_IN_FRAMES: usize = 180; //60fps :D

    pub fn config(&self) -> &ConfigRef {
        &self.config
    }

    pub fn new<P: AsRef<OsStr>>(path: P) -> FullSetupBuilder {
        FullSetupBuilder {
            path: PathBuf::from(path.as_ref()),
            config: None,
            files: vec![],
            size: FullSetupBuilder::DEFAULT_MOCK_OUTPUT_SIZE,
            recording: false,
            step_frame: false,
            frame_based_wait: false,
        }
    }

    // TODO remove
    pub fn wait_frame(&mut self) -> bool {
        let mut res = false;
        select! {
            recv(self.output_receiver) -> frame_res => {
                match frame_res {
                    Ok(frame) => {
                        self.last_frame = Some(frame);
                        res = true;
                    }
                    Err(e) => {
                        error!("failed retrieving frame : {:?}", e);
                    }
                }
            },
            default(Self::DEFAULT_TIMEOUT) => {
                error!("timeout");
            },
        }

        while let Ok(bo) = self.output_receiver.try_recv() {
            self.last_frame = Some(bo);
            res = true;
        }

        res
    }

    pub fn navcomp_pilot(&self) -> &MockNavCompProviderPilot {
        &self.mock_navcomp_pilot
    }

    pub fn get_frame(&self) -> Option<&MetaOutputFrame> {
        self.last_frame.as_ref()
    }

    pub fn is_editor_opened(&self) -> bool {
        self.last_frame.as_ref().unwrap().get_editors().next().is_some()
    }

    pub fn is_no_editor_opened(&self) -> bool {
        self.last_frame.as_ref().unwrap().get_no_editor().is_some()
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

    pub fn get_first_editor(&self) -> Option<EditorInterpreter<'_>> {
        self.last_frame.as_ref().map(|frame| {
            frame.get_editors().next()
        }).flatten()
    }

    pub fn get_file_tree_view(&self) -> Option<TreeViewInterpreter<'_>> {
        self.last_frame.as_ref().map(|frame| {
            frame.get_meta_by_type(crate::mocks::full_setup::tree_view::tree_view::TYPENAME)
                .filter(|meta| meta.rect.pos == XY::ZERO)
                .next()
                .map(|meta| TreeViewInterpreter::new(meta, frame))
        }).flatten()
    }

    pub fn get_code_results_view(&self) -> Option<CodeResultsViewInterpreter<'_>> {
        self.last_frame.as_ref().map(|frame| {
            frame.get_code_results_view()
        }).flatten()
    }

    pub fn get_fuzzy_search(&self) -> Option<FuzzySearchInterpreter> {
        self.last_frame.as_ref().map(|frame| frame.get_fuzzy_search()).flatten()
    }

    pub fn send_input(&self, ie: InputEvent) -> bool {
        self.input_sender.send(ie).is_ok()
    }

    pub fn send_key(&self, key: Key) -> bool {
        self.input_sender.send(InputEvent::KeyInput(key)).is_ok()
    }

    pub fn type_in(&self, text: &str) -> bool {
        let mut res = true;
        for c in text.chars() {
            res &= self.send_input(InputEvent::KeyInput(Keycode::Char(c).to_key()));
            if !res {
                break;
            }
        }

        res
    }

    /*
    waits with default timeout for condition F to be satisfied, returns whether that happened or not
     */
    pub fn wait_for<F: Fn(&FullSetup) -> bool>(&mut self, condition: F) -> bool {
        // maybe it's already true?
        if self.last_frame.as_ref().is_some() {
            if condition(&self) {
                return true;
            }
        }

        let mut waited_frames: usize = 0;

        /*
        When self.frame_based_wait == false, we wait at most DEFAULT_TIMEOUT for matching frame.
        Otherwise, we wait up to DEFAULT_TIMEOUT_IN_FRAMES frames, before returning false.
        The latter setting is designed for debugging, in continous integration it should be off.
         */

        if !self.frame_based_wait {
            loop {
                select! {
                    recv(self.output_receiver) -> frame_res => {
                        match frame_res {
                            Ok(frame) => {
                                self.last_frame = Some(frame);
                                if condition(&self) {
                                    return true;
                                }
                                debug!("no hit on condition");
                            }
                            Err(e) => {
                                error!("error receiving frame: {:?}", e);
                                return false;
                            }
                        }
                    },
                    default(Self::DEFAULT_TIMEOUT) => {
                        error!("timeout, making screenshot.");
                        self.screenshot();
                        return false;
                    }
                }
            }
        } else {
            warn!("TEST WAIT-TIMEOUT IS DISABLED");
            loop {
                select! {
                    recv(self.output_receiver) -> frame_res => {
                        match frame_res {
                            Ok(frame) => {
                                self.last_frame = Some(frame);
                                if condition(&self) {
                                    return true;
                                }
                                debug!("no hit on condition");
                            }
                            Err(e) => {
                                error!("error receiving frame: {:?}", e);
                                return false;
                            }
                        }
                    }
                }
                waited_frames += 1;
                if waited_frames >= Self::DEFAULT_TIMEOUT_IN_FRAMES {
                    error!("waited {} frames to no avail", waited_frames);
                    return false;
                }
            }
        }
    }

    pub fn screenshot(&self) -> bool {
        self.last_frame.as_ref().map(|frame| screenshot(&frame.buffer)).unwrap_or(false)
    }
}

pub struct FinishedFullSetupRun {
    pub fsf: FsfRef,
    pub last_frame: Option<MetaOutputFrame>,
    pub clipboard: ClipboardRef,
}

impl FinishedFullSetupRun {
    pub fn screenshot(&self) -> bool {
        self.last_frame.as_ref().map(|frame| screenshot(&frame.buffer)).unwrap_or(false)
    }
}