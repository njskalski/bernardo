use std::sync::{Arc, RwLock};

use crate::config::config::{Config, ConfigRef};
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::mock_fs::MockFS;
use crate::gladius::providers::Providers;
use crate::mocks::mock_clipboard::MockClipboard;
use crate::mocks::mock_navcomp_loader::MockNavcompLoader;
use crate::mocks::mock_navcomp_provider::{MockCompletionMatcher, MockNavCompEvent, MockNavCompProviderPilot, MockSymbolMatcher};
use crate::tsw::language_set::LanguageSet;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::widgets::editor_widget::label::labels_provider::LabelsProviderRef;

pub struct SideChannels {
    pub navcomp_pilot: MockNavCompProviderPilot,
}

pub struct MockProvidersBuilder {
    config_op: Option<Config>,
    fsf_op: Option<FsfRef>,
    label_providers: Vec<LabelsProviderRef>,
    theme_op: Option<Theme>,
}

pub struct BuildResult {
    pub providers: Providers,
    pub side_channels: SideChannels,
}

impl Default for MockProvidersBuilder {
    fn default() -> Self {
        MockProvidersBuilder {
            config_op: None,
            fsf_op: None,
            label_providers: vec![],
            theme_op: None,
        }
    }
}

impl MockProvidersBuilder {
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self = MockProvidersBuilder {
            theme_op: Some(theme),
            ..self
        };
        self
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self = MockProvidersBuilder {
            config_op: Some(config),
            ..self
        };
        self
    }

    pub fn with_label_provider(mut self, label_provider: LabelsProviderRef) -> Self {
        self.label_providers.push(label_provider);
        self
    }

    pub fn build(self) -> BuildResult {
        let config: ConfigRef = Arc::new(self.config_op.unwrap_or(Config::default()));
        let fsfref = self.fsf_op.unwrap_or(FsfRef::new(MockFS::new("/")));
        let clipboard: ClipboardRef = MockClipboard::default().into_clipboardref();
        let theme = self.theme_op.unwrap_or(Theme::default());
        let tree_sitter = Arc::new(TreeSitterWrapper::new(LanguageSet::full()));

        let (mock_navcomp_event_sender, mock_navcomp_event_recvr) = crossbeam_channel::unbounded::<MockNavCompEvent>();
        let comp_matcher: Arc<RwLock<Vec<MockCompletionMatcher>>> = Arc::new(RwLock::new(Vec::new()));
        let symbol_matcher: Arc<RwLock<Vec<MockSymbolMatcher>>> = Arc::new(RwLock::new(Vec::new()));

        let navcomp_loader = MockNavcompLoader::new(mock_navcomp_event_sender, comp_matcher.clone(), symbol_matcher.clone());
        let label_providers = self.label_providers;

        let providers = Providers::new(
            config,
            fsfref,
            clipboard,
            theme,
            tree_sitter,
            Arc::new(Box::new(navcomp_loader)),
            label_providers,
        );

        BuildResult {
            providers,
            side_channels: SideChannels {
                navcomp_pilot: MockNavCompProviderPilot::new(mock_navcomp_event_recvr, comp_matcher, symbol_matcher),
            },
        }
    }
}
