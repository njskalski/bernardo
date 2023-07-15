use std::collections::HashMap;
use std::rc::Rc;

use log::{debug, error, warn};
use uuid::Uuid;

use crate::{subwidget, unpack_or, unpack_or_e};
use crate::config::theme::Theme;
use crate::experiments::buffer_register::OpenResult;
use crate::experiments::filename_to_language::filename_to_language;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::path::SPath;
use crate::fs::read_error::ReadError;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::loading_state::LoadingState;
use crate::io::output::Output;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::xy::XY;
use crate::promise::promise::PromiseState;
use crate::text::buffer_state::BufferState;
use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
use crate::w7e::navcomp_group::NavCompGroupRef;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::code_results_view::code_results_provider::CodeResultsProvider;
use crate::widgets::code_results_view::code_results_widget::CodeResultsView;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::fuzzy_search::fsf_provider::{FsfProvider, SPathMsg};
use crate::widgets::fuzzy_search::fuzzy_search::{DrawComment, FuzzySearchWidget};
use crate::widgets::fuzzy_search::item_provider::ItemsProvider;
use crate::widgets::main_view::display::MainViewDisplay;
use crate::widgets::main_view::display_fuzzy::DisplayItem;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::spath_tree_view_node::FileTreeNode;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;
use crate::widgets::with_scroll::WithScroll;

pub type BufferId = Uuid;

pub enum HoverItem {
    FuzzySearch(WithScroll<FuzzySearchWidget>),
}

// TODO start indexing documents with DocumentIdentifier as opposed to usize

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct DocumentIdentifier {
    pub buffer_id: BufferId,
    pub file_path: Option<SPath>,
}

impl DocumentIdentifier {
    pub fn new_unique() -> DocumentIdentifier {
        DocumentIdentifier {
            buffer_id: Uuid::new_v4(),
            file_path: None,
        }
    }

    pub fn with_file_path(self, ff: SPath) -> Self {
        Self {
            file_path: Some(ff),
            ..self
        }
    }
}

pub struct MainView {
    wid: WID,
    providers: Providers,
    /*
    I use a simplified "display state" model, not the GenericFocusGroup approach, to see how much effort the Generic one saves.
    caveat: whenever focusing on editor, make sure to set curr_editor_index as well. It's a temporary solution, so I don't wrap it.
     */
    display_state: Option<DisplayState<MainView>>,

    // TODO PathBuf -> WrappedRcPath? See profiler.
    tree_widget: WithScroll<TreeViewWidget<SPath, FileTreeNode>>,

    no_editor: NoEditorWidget,
    displays: Vec<MainViewDisplay>,
    display_idx: usize,

    hover: Option<HoverItem>,
}

impl MainView {
    pub const MIN_SIZE: XY = XY::new(32, 10);

    pub fn create_new_display_for_code_results(&mut self, data_provider: Box<dyn CodeResultsProvider>) -> Result<usize, ()> {
        self.displays.push(
            MainViewDisplay::ResultsView(
                CodeResultsView::new(self.providers.clone(),
                                     data_provider,
                )
            )
        );

        let res = self.displays.len() - 1;

        Ok(res)
    }

    pub fn create_new_editor_for_file(&mut self, ff: &SPath) -> Result<usize, ReadError> {
        // TODO this should return some other error, but they are swallowed anyway
        let mut register_lock = unpack_or_e!(self.providers.buffer_register().try_write().ok(), Err(ReadError::FileNotFound), "failed to lock register");
        let OpenResult {
            buffer_shared_ref, opened
        } = register_lock.open_file(&self.providers, ff);
        let mut buffer_shared_ref = buffer_shared_ref?;

        if let Some(mut buffer_lock) = buffer_shared_ref.lock_rw() {
            buffer_lock.set_lang(filename_to_language(&ff));
        }

        self.displays.push(
            MainViewDisplay::Editor(
                EditorView::new(self.providers.clone(),
                                buffer_shared_ref,
                ).with_path_op(
                    ff.parent()
                ),
            )
        );

        let res = self.displays.len() - 1;

        Ok(res)
    }


    fn get_curr_display_ptr(&self) -> SubwidgetPointer<Self> {
        if self.display_idx >= self.displays.len() {
            if self.display_idx > 0 {
                error!("current editor points further than number opened editors!");
                return subwidget!(Self.tree_widget);
            }

            subwidget!(Self.no_editor)
        } else {
            let idx1 = self.display_idx;
            let idx2 = self.display_idx;
            SubwidgetPointer::new(
                Box::new(move |s: &Self| { s.displays.get(idx1).map(|w| w.get_widget()).unwrap_or(&s.no_editor) }),
                Box::new(move |s: &mut Self| { s.displays.get_mut(idx2).map(|w| w.get_widget_mut()).unwrap_or(&mut s.no_editor) }),
            )
        }
    }


    pub fn get_display_list_provider(&self) -> Box<dyn ItemsProvider> {
        Box::new(self.displays.iter().enumerate().map(|(idx, display)| {
            match display {
                MainViewDisplay::Editor(editor) => {
                    let text = match editor.get_path() {
                        None => {
                            format!("unnamed file #{}", idx)
                        }
                        Some(path) => {
                            path.label().to_string()
                        }
                    };

                    // TODO unnecessary Rc over new
                    DisplayItem::new(idx, Rc::new(text))
                }
                MainViewDisplay::ResultsView(result) => {
                    let text = result.get_text().clone();

                    DisplayItem::new(idx, text.into())
                }
            }
        }).collect::<Vec<_>>()
        )
    }

    fn get_editor_idx_for(&self, ff: &SPath) -> Option<usize> {
        let register = unpack_or_e!(self.providers.buffer_register().try_read().ok(), None, "failed locking register");
        let buffer_shared_ref = unpack_or!(register.get_buffer_ref_from_path(ff), None, "no buffer for path");

        for (idx, display) in self.displays.iter().enumerate() {
            match display {
                MainViewDisplay::Editor(editor) => {
                    if *editor.get_buffer_ref() == buffer_shared_ref {
                        return Some(idx);
                    }
                }
                MainViewDisplay::ResultsView(_) => {}
            }
        }
        None
    }

    fn get_hover_rect(output_size: XY, visible_rect: Rect) -> Option<Rect> {
        if output_size >= XY::new(10, 8) {
            let margin = output_size / 10;
            let res = Rect::new(margin,
                                output_size - margin * 2,
            );
            Some(res)
        } else {
            None
        }
    }

    pub fn new(providers: Providers
    ) -> MainView {
        let root = providers.fsf().root();
        let tree = TreeViewWidget::new(FileTreeNode::new(root.clone()))
            .with_on_flip_expand(|widget| {
                let (_, item) = widget.get_highlighted();

                Some(Box::new(MainViewMsg::TreeExpandedFlip {
                    expanded: widget.is_expanded(item.id()),
                    item: item.spath().clone(),
                }))
            })
            .with_on_select_hightlighted(|widget| {
                let (_, item) = widget.get_highlighted();
                Some(Box::new(MainViewMsg::TreeSelected {
                    item: item.spath().clone(),
                }))
            });

        MainView {
            wid: get_new_widget_id(),
            providers: providers.clone(),
            display_state: None,
            tree_widget: WithScroll::new(ScrollDirection::Both, tree),
            displays: Vec::new(),
            no_editor: NoEditorWidget::default(),
            display_idx: 0,
            hover: None,
        }
    }

    fn open_empty_editor_and_focus(&mut self) {
        let buffer = if let Some(mut buffer_register) = self.providers.buffer_register().try_write().ok() {
            buffer_register.open_new_file(&self.providers)
        } else {
            error!("failed to acquire register lock");
            return;
        };

        self.displays.push(
            MainViewDisplay::Editor(
                EditorView::new(
                    self.providers.clone(),
                    buffer,
                )
            )
        );

        self.display_idx = self.displays.len() - 1;
        self.set_focus_to_default();
    }

    pub fn open_file(&mut self, ff: SPath) -> bool {
        debug!("opening file {:?}", ff);

        if let Some(idx) = self.get_editor_idx_for(&ff) {
            self.display_idx = idx;
            self.set_focused(self.get_default_focused());
            true
        } else {
            self.create_new_editor_for_file(&ff)
                .map(|idx| {
                    self.display_idx = idx;
                    self.set_focused(self.get_default_focused());
                }).is_ok()
        }
    }

    fn open_fuzzy_buffer_list_and_focus(&mut self) {
        self.hover = Some(
            HoverItem::FuzzySearch(
                WithScroll::new(
                    ScrollDirection::Vertical,
                    FuzzySearchWidget::new(
                        |_| Some(Box::new(MainViewMsg::CloseHover)),
                        Some(self.providers.clipboard().clone()),
                    ).with_provider(
                        self.get_display_list_provider()
                    ).with_draw_comment_setting(DrawComment::Highlighted))
            )
        );
        self.set_focus_to_hover();
    }

    fn open_fuzzy_search_in_files_and_focus(&mut self) {
        self.hover = Some(
            HoverItem::FuzzySearch(
                WithScroll::new(
                    ScrollDirection::Vertical,
                    FuzzySearchWidget::new(
                        |_| Some(Box::new(MainViewMsg::CloseHover)),
                        Some(self.providers.clipboard().clone()),
                    ).with_provider(
                        Box::new(FsfProvider::new(self.providers.fsf().clone()).with_ignores_filter())
                    ).with_draw_comment_setting(DrawComment::Highlighted),
                ),
            )
        );
        self.set_focus_to_hover();
    }

    fn set_focus_to_default(&mut self) {
        let ptr = self.get_curr_display_ptr();
        self.set_focused(ptr);
    }

    fn set_focus_to_hover(&mut self) {
        let ptr_to_hover = SubwidgetPointer::<Self>::new(
            Box::new(|s: &MainView| {
                let hover_present = s.hover.is_some();
                if hover_present {
                    match s.hover.as_ref().unwrap() { HoverItem::FuzzySearch(fs) => fs as &dyn Widget }
                } else {
                    error!("failed to unwrap hover widget!");
                    s.get_default_focused().get(s)
                }
            }),
            Box::new(|s: &mut MainView| {
                let hover_present = s.hover.is_some();
                if hover_present {
                    match s.hover.as_mut().unwrap() { HoverItem::FuzzySearch(fs) => fs as &mut dyn Widget }
                } else {
                    error!("failed to unwrap hover widget!");
                    s.get_default_focused().get_mut(s)
                }
            }),
        );

        self.set_focused(ptr_to_hover);
    }

    pub fn with_empty_editor(mut self) -> Self {
        self.open_empty_editor_and_focus();
        self
    }

    // pub fn set_search_result(&mut self, crv_op: Option<CodeResultsView>) {
    //     // self.crv_op = crv_op;
    // }
}

impl Widget for MainView {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "main_view"
    }

    fn prelayout(&mut self) {
        self.complex_prelayout();
    }

    fn full_size(&self) -> XY {
        // TODO delegate to complex_layout?
        Self::MIN_SIZE
    }

    fn layout(&mut self, output_size: XY, visible_rect: Rect) {
        self.complex_layout(output_size, visible_rect)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug!("main_view.on_input {:?}", input_event);

        let config = self.providers.config();

        return match input_event {
            InputEvent::FocusUpdate(focus_update) if self.will_accept_focus_update(focus_update) => {
                MainViewMsg::FocusUpdateMsg(focus_update).someboxed()
            }
            InputEvent::KeyInput(key) if key == config.keyboard_config.global.new_buffer => {
                MainViewMsg::OpenNewFile.someboxed()
            }
            InputEvent::KeyInput(key) if key == config.keyboard_config.global.fuzzy_file => {
                MainViewMsg::OpenFuzzyFiles.someboxed()
            }
            InputEvent::KeyInput(key) if key == config.keyboard_config.global.browse_buffers => {
                if self.displays.is_empty() {
                    debug!("ignoring browse_buffers request - no displays open.");
                    None
                } else {
                    MainViewMsg::OpenFuzzyBuffers.someboxed()
                }
            }
            _ => {
                debug!("input {:?} NOT consumed", input_event);
                None
            }
        };
    }

    fn update(&mut self, mut msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("main_view.update {:?}", msg);

        if let Some(main_view_msg) = msg.as_msg_mut::<MainViewMsg>() {
            return match main_view_msg {
                MainViewMsg::FocusUpdateMsg(focus_update) => {
                    if !self.update_focus(*focus_update) {
                        error!("failed to accept focus update")
                    }

                    None
                }
                MainViewMsg::TreeExpandedFlip { .. } => {
                    None
                }
                MainViewMsg::TreeSelected { item } => {
                    if !self.open_file(item.clone()) {
                        error!("failed open_file");
                    }

                    None
                }
                MainViewMsg::OpenFuzzyFiles => {
                    self.open_fuzzy_search_in_files_and_focus();
                    None
                }
                MainViewMsg::CloseHover => {
                    if self.hover.is_none() {
                        error!("expected self.hover to be not None.");
                    }

                    self.hover = None;
                    None
                }
                MainViewMsg::OpenFuzzyBuffers => {
                    self.open_fuzzy_buffer_list_and_focus();
                    None
                }
                MainViewMsg::OpenNewFile => {
                    self.open_empty_editor_and_focus();
                    None
                }
                MainViewMsg::FuzzyBuffersHit { pos } => {
                    if *pos >= self.displays.len() {
                        error!("received FuzzyBufferHit for an index {} and len is {}, ignoring", pos, self.displays.len());
                    } else {
                        self.display_idx = *pos;
                    }
                    // removing the dialog
                    self.hover = None;
                    self.set_focus_to_default();

                    None
                }
                MainViewMsg::FindReferences { ref mut promise_op } => {
                    if let Some(promise) = promise_op.take() {
                        match self.create_new_display_for_code_results(
                            Box::new(promise)
                        ) {
                            Ok(idx) => {
                                self.display_idx = idx;
                                self.set_focus_to_default();
                            }
                            Err(_) => {
                                error!("failed find references");
                            }
                        }
                    } else {
                        warn!("find reference with empty promise")
                    }
                    None
                }
                _ => {
                    warn!("unprocessed event {:?}", main_view_msg);
                    None
                }
            };
        };

        if let Some(fuzzy_file_msg) = msg.as_msg::<SPathMsg>() {
            return match fuzzy_file_msg {
                SPathMsg::Hit(file_front) => {
                    if file_front.is_file() {
                        self.open_file(file_front.clone());
                        self.hover = None;
                        None
                    } else if file_front.is_dir() {
                        if !self.tree_widget.internal_mut().expand_path(file_front) {
                            error!("failed to set path")
                        }
                        self.hover = None;
                        None
                    } else {
                        error!("ff {:?} is neither file nor dir!", file_front);
                        None
                    }
                }
            };
        }

        warn!("expecetd MainViewMsg | FuzzyFileMsg, got {:?}", msg);
        None
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

impl ComplexWidget for MainView {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        let left_column = LeafLayout::new(subwidget!(Self.tree_widget)).boxed();
        let right_column = LeafLayout::new(self.get_curr_display_ptr()).boxed();

        let bg_layout = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0),
                  left_column)
            .with(SplitRule::Proportional(5.0),
                  right_column,
            );

        let res = if let Some(hover) = &self.hover {
            match hover {
                HoverItem::FuzzySearch(_fuzzy) => {
                    let hover = LeafLayout::new(SubwidgetPointer::new(
                        Box::new(|s: &Self| {
                            match s.hover.as_ref().unwrap() {
                                HoverItem::FuzzySearch(fs) => fs,
                            }
                        }),
                        Box::new(|s: &mut Self| {
                            match s.hover.as_mut().unwrap() {
                                HoverItem::FuzzySearch(fs) => fs,
                            }
                        }),
                    )).boxed();

                    HoverLayout::new(
                        bg_layout.boxed(),
                        hover,
                        Box::new(Self::get_hover_rect),
                        true,
                    ).boxed()
                }
            }
        } else {
            bg_layout.boxed()
        };

        res
    }

    fn get_default_focused(&self) -> SubwidgetPointer<MainView> {
        self.get_curr_display_ptr()
    }

    fn set_display_state(&mut self, display_state: DisplayState<MainView>) {
        self.display_state = Some(display_state);
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<MainView>> {
        self.display_state.as_ref()
    }

    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
        self.display_state.as_mut()
    }
}