use std::cmp::max;

use log::{debug, error, warn};

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::xy::XY;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::big_list::big_list_widget::BigList;
use crate::widgets::code_result_avatar::widget::CodeResultAvatarWidget;
use crate::widgets::code_results_view::code_results_msg::CodeResultsMsg;
use crate::widgets::code_results_view::code_results_provider::CodeResultsProvider;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::main_view::main_view::DocumentIdentifier;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::text_widget::TextWidget;
use crate::widgets::with_scroll::with_scroll::WithScroll;
use crate::{subwidget, unpack_or_e, unpack_unit_e};

pub struct CodeResultsView {
    wid: WID,

    label: TextWidget,
    item_list: WithScroll<BigList<CodeResultAvatarWidget>>,

    //providers
    data_provider: Box<dyn CodeResultsProvider>,

    providers: Providers,

    //
    display_state: Option<DisplayState<CodeResultsView>>,
}

impl CodeResultsView {
    pub const TYPENAME: &'static str = "code_results";
    pub const MIN_WIDTH: u16 = 20;

    pub fn new(providers: Providers, data_provider: Box<dyn CodeResultsProvider>) -> Self {
        Self {
            wid: get_new_widget_id(),
            label: TextWidget::new(Box::new("no description")).with_size_policy(SizePolicy::MATCH_LAYOUTS_WIDTH),
            item_list: WithScroll::new(
                ScrollDirection::Horizontal,
                BigList::new(vec![]).with_size_policy(SizePolicy::MATCH_LAYOUT),
            ),
            data_provider,
            providers,
            display_state: None,
        }
    }

    pub fn get_description(&self) -> String {
        self.label.get_text()
    }

    pub fn get_selected_item(&self) -> &CodeResultAvatarWidget {
        self.item_list.internal().get_selected_item()
    }

    pub fn get_selected_item_mut(&mut self) -> &mut CodeResultAvatarWidget {
        self.item_list.internal_mut().get_selected_item_mut()
    }

    pub fn get_selected_doc_id(&self) -> DocumentIdentifier {
        // TODO unwrap
        self.get_selected_item()
            .get_buffer_ref()
            .lock()
            .unwrap()
            .get_document_identifier()
            .clone()
    }

    fn on_hit(&self) -> Option<Box<dyn AnyMsg>> {
        let editor = self.get_selected_item();
        let editor_widget_id = editor.get_editor_widget().id();
        let buffer_lock = editor.get_buffer_ref().lock()?;
        let cursors = buffer_lock.cursors(editor_widget_id);
        let single_cursor = unpack_or_e!(cursors.map(|cs| cs.as_single()).flatten(), None, "can't single the cursor");

        MainViewMsg::OpenFile {
            file: buffer_lock.get_document_identifier().clone(),
            position_op: Some(single_cursor),
        }
        .someboxed()
    }
}

impl Widget for CodeResultsView {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }
    fn prelayout(&mut self) {
        let was_empty = self.item_list.internal().is_empty();
        self.data_provider.poll();

        self.label.set_text(self.data_provider.description());

        {
            let mut buffer_register_lock = unpack_unit_e!(
                self.providers.buffer_register().try_write().ok(),
                "failed to acquire buffer register",
            );

            for symbol in self.data_provider.items().skip(self.item_list.internal().items().count()) {
                // TODO unwrap
                let buffer_state_ref = buffer_register_lock
                    .open_file(&self.providers, &symbol.path)
                    .buffer_shared_ref
                    .unwrap();

                let mut edit_view = EditorView::new(self.providers.clone(), buffer_state_ref)
                    .with_readonly()
                    .with_ignore_input_altogether();

                let cursor_set = symbol.range.as_cursor_set();

                if edit_view.get_internal_widget_mut().set_cursors(cursor_set) == false {
                    error!("failed setting cursor set for file {}", symbol.path);
                }

                let item = CodeResultAvatarWidget::new(edit_view);

                self.item_list.internal_mut().add_item(SplitRule::Fixed(5), item)
            }
        } // to drop buffer_register_lock

        let is_empty = self.item_list.internal().is_empty();

        if was_empty && !is_empty {
            self.set_focused(subwidget!(Self.item_list));
        }

        self.complex_prelayout();
    }

    fn size_policy(&self) -> SizePolicy {
        SizePolicy::MATCH_LAYOUT
    }

    fn full_size(&self) -> XY {
        let item_min_size = self.item_list.full_size();
        XY::new(max(Self::MIN_WIDTH, item_min_size.x), 1 + item_min_size.y)
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.complex_layout(screenspace)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug!("{} input {:?}", self.typename(), input_event);

        match input_event {
            InputEvent::KeyInput(key) if key == Keycode::Enter.to_key() => CodeResultsMsg::Hit.someboxed(),
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<CodeResultsMsg>();
        if our_msg.is_none() {
            warn!("expecetd CodeResultsMsg, got {:?}", msg);
            return None;
        }

        match our_msg.unwrap() {
            CodeResultsMsg::Hit => self.on_hit(),
        }
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        self.complex_get_focused()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        self.complex_get_focused_mut()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        {
            output.emit_metadata(crate::io::output::Metadata {
                id: self.wid,
                typename: self.typename().to_string(),
                rect: crate::primitives::rect::Rect::from_zero(output.size()),
                focused,
            });
        }

        self.complex_render(theme, focused, output)
    }
}

impl ComplexWidget for CodeResultsView {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Fixed(1), LeafLayout::new(subwidget!(Self.label)).boxed())
            .with(SplitRule::Proportional(1.0f32), LeafLayout::new(subwidget!(Self.item_list)).boxed())
            .boxed()
    }

    fn get_default_focused(&self) -> SubwidgetPointer<Self> {
        if self.item_list.internal().items().next().is_none() {
            subwidget!(Self.label)
        } else {
            subwidget!(Self.item_list)
        }
    }

    fn set_display_state(&mut self, display_state: DisplayState<Self>) {
        self.display_state = Some(display_state);
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<Self>> {
        self.display_state.as_ref()
    }

    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
        self.display_state.as_mut()
    }
}
