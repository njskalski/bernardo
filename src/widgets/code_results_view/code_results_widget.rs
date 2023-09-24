use std::cmp::max;
use std::collections::HashSet;

use log::{debug, error, warn};

use crate::{subwidget, unpack_or, unpack_or_e};
use crate::config::theme::Theme;
use crate::cursor::cursor_set::CursorSet;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::{Metadata, Output};
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::xy::XY;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::big_list::big_list_widget::BigList;
use crate::widgets::code_results_view::code_results_msg::CodeResultsMsg;
use crate::widgets::code_results_view::code_results_provider::CodeResultsProvider;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::main_view::main_view::DocumentIdentifier;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::text_widget::TextWidget;
use crate::widgets::with_scroll::with_scroll::WithScroll;

pub struct CodeResultsView {
    wid: WID,

    label: TextWidget,
    item_list: WithScroll<BigList<EditorWidget>>,

    //providers
    data_provider: Box<dyn CodeResultsProvider>,
    failed_ids: HashSet<usize>,

    providers: Providers,

    //
    display_state: Option<DisplayState<CodeResultsView>>,
}

impl CodeResultsView {
    pub const TYPENAME: &'static str = "code_results";
    pub const MIN_WIDTH: u16 = 20;

    pub fn new(
        providers: Providers,
        data_provider: Box<dyn CodeResultsProvider>,
    ) -> Self {
        Self {
            wid: get_new_widget_id(),
            label: TextWidget::new(Box::new("no description")).with_size_policy(SizePolicy::MATCH_LAYOUTS_WIDTH),
            item_list: WithScroll::new(ScrollDirection::Horizontal,
                                       BigList::new(vec![])
                                           .with_size_policy(SizePolicy::MATCH_LAYOUT),
            ),
            data_provider,
            failed_ids: HashSet::new(),
            providers,
            display_state: None,
        }
    }

    pub fn get_text(&self) -> String {
        self.label.get_text()
    }

    pub fn get_selected_item(&self) -> &EditorWidget {
        self.item_list.internal().get_selected_item()
    }

    pub fn get_selected_item_mut(&mut self) -> &mut EditorWidget {
        self.item_list.internal_mut().get_selected_item_mut()
    }

    pub fn get_selected_doc_id(&self) -> DocumentIdentifier {
        // TODO unwrap
        self.get_selected_item().get_buffer().lock().unwrap().get_document_identifier().clone()
    }

    fn on_hit(&self) -> Option<Box<dyn AnyMsg>> {
        let editor = self.get_selected_item();
        let editor_widget_id = editor.id();
        let buffer = unpack_or_e!(editor.get_buffer().lock(), None, "can't lock buffer");
        let single_cursor = unpack_or!(buffer.cursors(editor_widget_id).map(|cs| cs.as_single()).flatten(), None, "can't single the cursor");

        MainViewMsg::OpenFile {
            file: buffer.get_document_identifier().clone(),
            position_op: single_cursor,
        }.someboxed()
    }
}

impl Widget for CodeResultsView {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn static_typename() -> &'static str where Self: Sized {
        Self::TYPENAME
    }
    fn prelayout(&mut self) {
        let was_empty = self.item_list.internal().is_empty();
        self.data_provider.poll();

        self.label.set_text(self.data_provider.description());

        {
            let mut buffer_register_lock = unpack_or_e!(self.providers.buffer_register().try_write().ok(), (), "failed to acquire buffer register");

            for (idx, symbol) in self.data_provider.items().enumerate().skip(self.item_list.internal().items().count()) {
                // TODO URGENT loading files should be moved out from here to some common place between this and Editor

                // this just skips failed loads. TODO add a "load error widget?"
                if self.failed_ids.contains(&idx) {
                    continue;
                }

                debug!("processing symgol usage {:?}", &symbol);

                let no_prefix = match symbol.path.strip_prefix("file://") {
                    None => {
                        error!("failed stripping prefix file:// from {}", &symbol.path);
                        self.failed_ids.insert(idx);
                        continue;
                    }
                    Some(np) => np,
                };

                // TODO two unwraps
                let root_path_buf = self.providers.fsf().root_path_buf().to_string_lossy().to_string() + "/";

                let in_workspace = match no_prefix.strip_prefix(&root_path_buf) {
                    None => {
                        error!("failed stripping prefix root_path from {}", &no_prefix);
                        self.failed_ids.insert(idx);
                        continue;
                    }
                    Some(np) => np,
                };

                let spath = match self.providers.fsf().descendant_checked(&in_workspace) {
                    None => {
                        error!("failed to get spath from {}", &in_workspace);
                        self.failed_ids.insert(idx);
                        continue;
                    }
                    Some(s) => s,
                };

                let open_result = buffer_register_lock.open_file(&self.providers, &spath);

                if open_result.buffer_shared_ref.is_err() {
                    error!("failed to load buffer {} because {}", spath, open_result.buffer_shared_ref.err().unwrap());
                    self.failed_ids.insert(idx);
                    continue;
                }

                let buffer_state_ref = open_result.buffer_shared_ref.unwrap();

                let cursor_set: CursorSet = match buffer_state_ref.lock_rw() {
                    None => {
                        error!("failed to lock buffer {}", spath);
                        self.failed_ids.insert(idx);
                        continue;
                    }
                    Some(mut buffer) => {
                        match symbol.stupid_range.0.to_real_cursor(&*buffer) {
                            None => {
                                error!("failed to cast StupidCursor to a real one");
                                self.failed_ids.insert(idx);
                                continue;
                            }
                            Some(first_cursor) => {
                                if open_result.opened {
                                    warn!("I will destroy cursor data, because issue #23 - we don't have multiple views properly implemented, sorry");
                                }

                                buffer.remove_history();

                                CursorSet::singleton(first_cursor)
                            }
                        }
                    }
                };

                let mut edit_widget = EditorWidget::new(
                    self.providers.clone(),
                    buffer_state_ref,
                ).with_readonly()
                    .with_ignore_input_altogether();

                if edit_widget.set_cursors(cursor_set) == false {
                    error!("failed setting cursor set, will not add this editor to list {}", spath);
                    self.failed_ids.insert(idx);
                    continue;
                }

                self.item_list.internal_mut().add_item(
                    SplitRule::Fixed(5),
                    edit_widget,
                )
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
            InputEvent::KeyInput(key) if key == Keycode::Enter.to_key() => {
                CodeResultsMsg::Hit.someboxed()
            }
            _ => None
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<CodeResultsMsg>();
        if our_msg.is_none() {
            warn!("expecetd CodeResultsMsg, got {:?}", msg);
            return None;
        }

        #[allow(unreachable_patterns)]
        return match our_msg.unwrap() {
            CodeResultsMsg::Hit => {
                self.on_hit()
            }
        };
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
            output.emit_metadata(
                Metadata {
                    id: self.wid,
                    typename: self.typename().to_string(),
                    rect: Rect::from_zero(output.size()),
                    focused,
                }
            );
        }

        self.complex_render(theme, focused, output)
    }
}

impl ComplexWidget for CodeResultsView {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        SplitLayout::new(SplitDirection::Vertical)
            .with(
                SplitRule::Fixed(1),
                LeafLayout::new(subwidget!(Self.label)).boxed(),
            )
            .with(
                SplitRule::Proportional(1.0f32),
                LeafLayout::new(subwidget!(Self.item_list)).boxed(),
            )
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