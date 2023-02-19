use std::cmp::max;
use std::collections::HashSet;
use std::rc::Rc;
use std::str::from_utf8;

use either::Left;
use log::{debug, error, warn};

use crate::config::theme::Theme;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::loading_state::LoadingState;
use crate::io::output::Output;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::cursor_set::Cursor;
use crate::primitives::cursor_set::CursorSet;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::subwidget;
use crate::text::buffer_state::BufferState;
use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::big_list::big_list_widget::BigList;
use crate::widgets::code_results_view::code_results_msg::CodeResultsMsg;
use crate::widgets::code_results_view::code_results_provider::CodeResultsProvider;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::main_view::main_view::DocumentIdentifier;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::text_widget::TextWidget;
use crate::widgets::with_scroll::WithScroll;

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
            label: TextWidget::new(Box::new("no description")),
            item_list: WithScroll::new(ScrollDirection::Vertical,
                                       BigList::new(vec![]),
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
}

impl Widget for CodeResultsView {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn prelayout(&mut self) {
        // TODO this method is stitched together from bullshit with ducttape. It's to be rewritten, after I figure out which items go well together.



        if self.data_provider.loading_state() == LoadingState::InProgress {
            self.data_provider.poll();
        }

        for (idx, symbol) in self.data_provider.items().enumerate().skip(self.item_list.internal().items().count()) {
            // TODO URGENT loading files should be moved out from here to some common place between this and Editor

            if self.failed_ids.contains(&idx) {
                continue;
            }

            let no_prefix = match symbol.path.strip_prefix("file://") {
                None => {
                    error!("failed stripping prefix from {}", &symbol.path);
                    self.failed_ids.insert(idx);
                    continue;
                }
                Some(np) => np,
            };

            // TODO two unwraps
            // let root = self.providers.fsf().root().0.as_path().unwrap().to_str().unwrap().to_string();
            let root_path_buf = self.providers.fsf().root_path_buf().to_string_lossy().to_string() + "/";

            let in_workspace = match no_prefix.strip_prefix(&root_path_buf) {
                None => {
                    error!("failed stripping prefix from {}", &no_prefix);
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

            let buffer_bytes = match self.providers.fsf().blocking_read_entire_file(&spath) {
                Ok(buf) => buf,
                Err(e) => {
                    error!("failed loading file {}, because {}", &spath, e);
                    self.failed_ids.insert(idx);
                    continue;
                }
            };

            let buffer_str = match from_utf8(&buffer_bytes) {
                Ok(s) => s,
                Err(e) => {
                    error!("failed loading file {}, because utf8 error {}", &spath, e);
                    self.failed_ids.insert(idx);
                    continue;
                }
            };

            let buffer_state = BufferState::full(
                Some(self.providers.tree_sitter().clone()),
                DocumentIdentifier::new_unique(),
            ).with_text(buffer_str);

            let first_cursor = match symbol.stupid_range.0.to_real_cursor(&buffer_state) {
                None => {
                    error!("failed to cast StupidCursor to a real one");
                    self.failed_ids.insert(idx);
                    continue;
                }
                Some(c) => c,
            };

            // TODO second_cursor?

            let buffer_state_ref = BufferSharedRef::new_from_buffer(buffer_state);

            let cursor_set = CursorSet::singleton(first_cursor);

            let mut edit_widget = EditorWidget::new(
                self.providers.clone(),
                None, // TODO add navcomp
                Some(buffer_state_ref),
            ).with_readonly()
                .with_ignore_input_altogether();

            if edit_widget.set_cursors(cursor_set) == false {
                error!("failed to set the cursor");
                self.failed_ids.insert(idx);
                continue;
            }

            self.item_list.internal_mut().add_item(
                SplitRule::Fixed(5),
                edit_widget,
            );
        }

        self.complex_prelayout();
    }

    fn size(&self) -> XY {
        let item_min_size = self.item_list.size();
        XY::new(max(Self::MIN_WIDTH, item_min_size.x), 1 + item_min_size.y)
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        self.complex_layout(sc)
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
                // TODO using first instead of "single"
                if let Some((doc_id, cursor)) = self.get_selected_item().get_buffer().lock().map(|lock| {
                    let doc_id = lock.get_document_identifier().clone();
                    let cursor = lock.cursors().first();

                    (doc_id, cursor)
                }) {
                    MainViewMsg::OpenFile {
                        file: doc_id.clone(),
                        position_op: cursor,
                    }.someboxed()
                } else {
                    error!("couldn't get item path");

                    None
                }
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
        subwidget!(Self.item_list)
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