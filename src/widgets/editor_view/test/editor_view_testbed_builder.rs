use crossbeam_channel::{Receiver, Sender};

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::fsf_ref::FsfRef;
use crate::io::input_event::InputEvent;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::mock_navcomp_provider::MockNavCompProviderPilot;
use crate::mocks::mock_output::MockOutput;
use crate::primitives::xy::XY;
use crate::text::buffer_state::BufferState;
use crate::tsw::lang_id::LangId;
use crate::widget::widget::Widget;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::editor_view::test::editor_view_testbed::EditorViewTestbed;
use crate::widgets::main_view::main_view::DocumentIdentifier;
use crate::widgets::tests::generic_widget_testbed_builder::GenericWidgetTestbedBuilder;

// pub struct EditorViewTestbedBuilder {
//     provider_builder: MockProvidersBuilder,
//     size: XY,
//     step_frame: bool,
// }

pub type EditorViewTestbedBuilder = GenericWidgetTestbedBuilder<EditorView, ()>;

impl EditorViewTestbedBuilder {
    pub const DEFAULT_MOCK_OUTPUT_SIZE: XY = XY::new(120, 36);

    // pub fn new(additional_data: ()) -> Self {
    //     EditorViewTestbedBuilder {
    //         size: Some(Self::DEFAULT_MOCK_OUTPUT_SIZE),
    //         providers: None,
    //         mock_nav_comp_provider: None,
    //         additional_data: (),
    //         theme: None,
    //         config: None,
    //         // step_frame: false,
    //         _phantom_data: Default::default(),
    //     }
    // }

    // pub fn with_config(self, config: Config) -> Self {
    //     Self {
    //         provider_builder: self.provider_builder.with_config(config),
    //         ..self
    //     }
    // }

    // pub fn with_theme(self, theme: Theme) -> Self {
    //     Self {
    //         provider_builder: self.provider_builder.with_theme(theme),
    //         ..self
    //     }
    // }

    // pub fn with_label_provider(mut self, provider: LabelsProviderRef) -> Self {
    //     Self {
    //         provider_builder: self.provider_builder.with_label_provider(provider),
    //         ..self
    //     }
    // }

    pub fn build(self) -> EditorViewTestbed {
        let size = self.size.unwrap_or(Self::DEFAULT_MOCK_OUTPUT_SIZE);
        let build_result = self.providers.build();

        let docid = DocumentIdentifier::new_unique();
        let buffer = BufferState::full(
            Some(build_result.providers.tree_sitter().clone()),
            docid,
            None,
            build_result.providers.config().global.tabs_to_spaces,
        )
        .with_lang(LangId::RUST)
        .into_bsr();

        let editor_view = EditorView::new(build_result.providers.clone(), buffer.clone());

        assert!(buffer
            .lock_rw()
            .unwrap()
            .text()
            .get_cursor_set(editor_view.get_internal_widget().id())
            .is_some());

        let (output, recv) = MockOutput::new(size, false, build_result.providers.theme().clone());

        EditorViewTestbed {
            widget: editor_view,

            additional_data: (),
            size,
            providers: build_result.providers,
            last_frame: None,
            mock_navcomp_pilot: Some(build_result.side_channels.navcomp_pilot),
            output,
            recv,
            last_msg: None,
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
