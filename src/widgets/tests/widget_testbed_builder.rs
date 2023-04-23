use std::sync::{Arc, RwLock};

use crossbeam_channel::{Receiver, Sender};

use crate::config::config::{Config, ConfigRef};
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::mock_fs::MockFS;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_clipboard::MockClipboard;
use crate::mocks::mock_input::MockInput;
use crate::mocks::mock_navcomp_loader::MockNavcompLoader;
use crate::mocks::mock_navcomp_provider::{MockCompletionMatcher, MockNavCompEvent, MockNavCompProviderPilot, MockSymbolMatcher};
use crate::mocks::mock_output::MockOutput;
use crate::primitives::xy::XY;
use crate::text::buffer_state::BufferState;
use crate::tsw::lang_id::LangId;
use crate::tsw::language_set::LanguageSet;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::main_view::main_view::DocumentIdentifier;
use crate::widgets::tests::editor_view_testbed::EditorViewTestbed;

pub struct SideChannels {
    pub navcomp_pilot: MockNavCompProviderPilot,
}

pub struct WidgetTestbedBuilder {
    size: XY,
    fsf: Option<Box<dyn FilesystemFront>>,
    config: Option<Config>,
    theme: Option<Theme>,
    step_frame: bool,
}

impl WidgetTestbedBuilder {
    pub const DEFAULT_MOCK_OUTPUT_SIZE: XY = XY::new(120, 36);

    pub fn new() -> Self {
        WidgetTestbedBuilder {
            size: Self::DEFAULT_MOCK_OUTPUT_SIZE,
            fsf: None,
            config: None,
            theme: None,
            step_frame: false,
        }
    }

    pub fn with_config(self, config: Config) -> Self {
        Self {
            config: Some(config),
            ..self
        }
    }

    pub fn with_size(self, size: XY) -> Self {
        Self {
            size,
            ..self
        }
    }

    pub fn with_theme(self, theme: Theme) -> Self {
        Self {
            theme: Some(theme),
            ..self
        }
    }

    pub fn with_step_frame(self, step_frame: bool) -> Self {
        Self {
            step_frame,
            ..self
        }
    }

    pub fn providers(self) -> (Providers, SideChannels) {
        let config: ConfigRef = Arc::new(self.config.unwrap_or(Config::default()));
        let fsfref: FsfRef = FsfRef::new(MockFS::new("/"));
        let clipboard: ClipboardRef = MockClipboard::default().into_clipboardref();
        let theme = self.theme.unwrap_or(Theme::default());
        let tree_sitter = Arc::new(TreeSitterWrapper::new(LanguageSet::full()));

        let (mock_navcomp_event_sender, mock_navcomp_event_recvr) = crossbeam_channel::unbounded::<MockNavCompEvent>();
        let comp_matcher: Arc<RwLock<Vec<MockCompletionMatcher>>> = Arc::new(RwLock::new(Vec::new()));
        let symbol_matcher: Arc<RwLock<Vec<MockSymbolMatcher>>> = Arc::new(RwLock::new(Vec::new()));

        let navcomp_loader = MockNavcompLoader::new(mock_navcomp_event_sender,
                                                    comp_matcher.clone(),
                                                    symbol_matcher.clone());

        (
            Providers::new(config, fsfref, clipboard, theme, tree_sitter, Arc::new(Box::new(navcomp_loader))),
            SideChannels {
                navcomp_pilot: MockNavCompProviderPilot::new(
                    mock_navcomp_event_recvr,
                    comp_matcher,
                    symbol_matcher,
                ),
            },
        )
    }

    pub fn build_editor(self) -> EditorViewTestbed {
        let size = self.size;
        let step_frame = self.step_frame;
        let (providers, side_channels) = self.providers();

        let (input, input_sender) = MockInput::new();
        let (output, output_receiver) = MockOutput::new(size, step_frame, providers.theme().clone());

        let docid = DocumentIdentifier::new_unique();
        let buffer = BufferState::full(Some(providers.tree_sitter().clone()), docid)
            .with_lang(LangId::RUST).into_bsr();

        let editor_view = EditorView::new(
            providers.clone(),
            buffer,
        );

        EditorViewTestbed {
            editor_view,
            input_sender,
            output_receiver,
            config: providers.config().clone(),
            clipboard: providers.clipboard().clone(),
            theme: providers.theme().clone(),
            last_frame: None,
            mock_navcomp_pilot: side_channels.navcomp_pilot,
        }
    }
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




