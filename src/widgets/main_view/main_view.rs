use std::path::Display;
use std::rc::Rc;

use log::{debug, error, warn};

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::filename_to_language::filename_to_language;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::path::SPath;
use crate::fs::read_error::ReadError;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::promise::promise::PromiseState;
use crate::subwidget;
use crate::text::buffer_state::BufferState;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::w7e::navcomp_group::NavCompGroupRef;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::code_results_view::code_results_provider::CodeResultsProvider;
use crate::widgets::code_results_view::code_results_widget::CodeResultsView;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::fuzzy_search::fsf_provider::{FsfProvider, SPathMsg};
use crate::widgets::fuzzy_search::fuzzy_search::{DrawComment, FuzzySearchWidget};
use crate::widgets::fuzzy_search::item_provider::ItemsProvider;
use crate::widgets::main_view::display::MainViewDisplay;
use crate::widgets::main_view::editor_group::EditorGroup;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::spath_tree_view_node::FileTreeNode;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;
use crate::widgets::with_scroll::WithScroll;

pub enum HoverItem {
    FuzzySearch(WithScroll<FuzzySearchWidget>),
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

    // TODO move to providers?
    nav_comp_group: NavCompGroupRef,

    no_editor: NoEditorWidget,
    displays: Vec<MainViewDisplay>,
    display_idx: usize,

    hover: Option<HoverItem>,
}

impl MainView {
    pub const MIN_SIZE: XY = XY::new(32, 10);

    pub fn new(providers: Providers,
               nav_comp_group: NavCompGroupRef,
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
            nav_comp_group,
            no_editor: NoEditorWidget::default(),
            display_idx: 0,
            hover: None,
        }
    }

    pub fn with_empty_editor(mut self) -> Self {
        self.open_empty_editor_and_focus();
        self
    }

    fn open_empty_editor_and_focus(&mut self) {
        self.displays.push(
            MainViewDisplay::Editor(
                EditorView::new(
                    self.providers.clone(),
                    self.nav_comp_group.clone(),
                )
            )
        );

        self.display_idx = self.displays.len() - 1;
        self.set_focused(self.get_default_focused())
    }

    fn get_hover_rect(sc: SizeConstraint) -> Option<Rect> {
        sc.as_finite().map(|finite_sc| {
            if finite_sc >= XY::new(10, 8) {
                let margin = finite_sc / 10;
                let res = Rect::new(margin,
                                    finite_sc - margin * 2,
                );
                Some(res)
            } else {
                None
            }
        }).flatten()
    }

    fn get_editor_idx_for(&self, ff: &SPath) -> Option<usize> {
        for (idx, display) in self.displays.iter().enumerate() {
            match display {
                MainViewDisplay::Editor(editor) => {
                    if let Some(cff) = editor.buffer_state().get_path() {
                        if cff == ff {
                            return Some(idx);
                        }
                    }
                }
                MainViewDisplay::ResultsView(_) => {}
            }
        }

        None
    }

    pub fn create_new_editor_for_file(&mut self, ff: SPath) -> Result<usize, ReadError> {
        let file_contents = ff.read_entire_file_to_rope()?;
        let lang_id_op = filename_to_language(&ff);

        let tree_sitter = self.providers.tree_sitter().clone();
        self.displays.push(
            MainViewDisplay::Editor(
                EditorView::new(self.providers.clone(),
                                self.nav_comp_group.clone(),
                ).with_buffer(
                    BufferState::full(Some(tree_sitter))
                        .with_text_from_rope(file_contents, lang_id_op)
                        .with_file_front(ff.clone())
                ).with_path_op(
                    ff.parent()
                ),
            )
        );

        let res = self.displays.len() - 1;

        Ok(res)
    }

    pub fn create_new_display_for_code_results(&mut self, data_provider: Box<dyn CodeResultsProvider>) -> Result<usize, ()> {
        self.displays.push(
            MainViewDisplay::ResultsView(
                CodeResultsView::new(self.providers.clone(),
                                     "blabla".into(),
                                     data_provider,
                )
            )
        );

        let res = self.displays.len() - 1;

        Ok(res)
    }


    pub fn open_file(&mut self, ff: SPath) -> bool {
        debug!("opening file {:?}", ff);

        if let Some(idx) = self.get_editor_idx_for(&ff) {
            self.display_idx = idx;
            self.set_focused(self.get_default_focused());
            true
        } else {
            self.create_new_editor_for_file(ff)
                .map(|idx| {
                    self.display_idx = idx;
                    self.set_focused(self.get_default_focused());
                }).is_ok()
        }
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
        self.set_focused_to_hover();
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
        self.set_focused_to_hover();
    }

    pub fn get_display_list_provider(&self) -> Box<dyn ItemsProvider> {
        Box::new(&self.displays)
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

    fn set_focused_to_hover(&mut self) {
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

    // fn set_focus_to_code_result_view(&mut self) {
    //     if self.crv_op.is_none() {
    //         error!("failed setting focus to code results view - no results");
    //         return;
    //     }
    //
    //     let ptr_to_crv_op = SubwidgetPointer::<Self>::new(
    //         Box::new(|s: &MainView| {
    //             let crv_present = s.crv_op.is_some();
    //             if crv_present {
    //                 s.crv_op.as_ref().unwrap() as &dyn Widget
    //             } else {
    //                 error!("failed to unwrap crv widget!");
    //                 s.get_default_focused().get(s)
    //             }
    //         }),
    //         Box::new(|s: &mut MainView| {
    //             let crv_present = s.crv_op.is_some();
    //             if crv_present {
    //                 s.crv_op.as_mut().unwrap() as &mut dyn Widget
    //             } else {
    //                 error!("failed to unwrap crv widget!");
    //                 s.get_default_focused().get_mut(s)
    //             }
    //         }),
    //     );
    //
    //     self.set_focused(ptr_to_crv_op);
    // }

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

    fn size(&self) -> XY {
        // TODO delegate to complex_layout?
        Self::MIN_SIZE
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        self.complex_layout(sc)
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
        // debug!("main_view.update {:?}", msg);

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

                    None
                }
                MainViewMsg::FindReferences { ref mut promise_op } => {
                    if let Some(mut promise) = promise_op.take() {
                        promise.update();
                        if promise.state() != PromiseState::Broken {
                            self.create_new_display_for_code_results(
                                Box::new(promise)
                            );
                            //TODO update focus
                        } else {
                            warn!("promise broken, throwing away.");
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

    fn render(&self, theme: &Theme, _focused: bool, output: &mut dyn Output) {
        self.complex_render(theme, _focused, output)
    }
}

impl ComplexWidget for MainView {
    fn get_layout(&self) -> Box<dyn Layout<Self>> {
        let left_column = LeafLayout::new(subwidget!(Self.tree_widget)).boxed();
        let right_column = if self.crv_op.is_none() {
            LeafLayout::new(self.get_curr_display_ptr()).boxed()
        } else {
            LeafLayout::new(SubwidgetPointer::new(
                Box::new(|s: &Self| s.crv_op.as_ref().unwrap()),
                Box::new(|s: &mut Self| s.crv_op.as_mut().unwrap()),
            )).boxed()
        };

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