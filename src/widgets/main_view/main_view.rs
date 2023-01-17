use std::rc::Rc;

use log::{debug, error, warn};

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::path::SPath;
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
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::w7e::navcomp_group::NavCompGroupRef;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::code_results_view::code_results_widget::CodeResultsView;
use crate::widgets::fuzzy_search::fsf_provider::{FsfProvider, SPathMsg};
use crate::widgets::fuzzy_search::fuzzy_search::{DrawComment, FuzzySearchWidget};
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

    editors: EditorGroup,

    //TODO
    crv_op: Option<CodeResultsView>,

    no_editor: NoEditorWidget,
    curr_editor_idx: usize,

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
            editors: EditorGroup::new(
                providers.config().clone(),
                nav_comp_group.clone(),
            ),
            crv_op: None,
            no_editor: NoEditorWidget::default(),
            curr_editor_idx: 0,
            hover: None,
        }
    }

    pub fn with_empty_editor(mut self) -> Self {
        self.open_empty_editor_and_focus();
        self
    }

    fn open_empty_editor_and_focus(&mut self) {
        let idx = self.editors.open_empty(
            self.providers.tree_sitter().clone(),
            self.providers.fsf().clone(),
            self.providers.clipboard().clone());
        self.curr_editor_idx = idx;
        self.set_focused(self.get_default_focused())
    }

    fn get_hover_rect(max_size: XY) -> Rect {
        let margin = max_size / 10;
        let res = Rect::new(margin,
                            max_size - margin * 2,
        );

        //edge case when... wtf
        if !(res.lower_right() >= max_size) {
            res.size.cut(SizeConstraint::simple(max_size));
        }

        res
    }


    pub fn open_file(&mut self, ff: SPath) -> bool {
        debug!("opening file {:?}", ff);

        if let Some(idx) = self.editors.get_if_open(&ff) {
            self.curr_editor_idx = idx;
            self.set_focused(self.get_default_focused());
            true
        } else {
            self.editors.open_file(self.providers.tree_sitter().clone(),
                                   ff,
                                   self.providers.clipboard().clone()).map(|idx| {
                self.curr_editor_idx = idx;
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
                        self.editors.get_buffer_list_provider()
                    ).with_draw_comment_setting(DrawComment::Highlighted))
            )
        );
        self.set_focused_to_hover();
    }

    fn get_curr_editor_ptr(&self) -> SubwidgetPointer<Self> {
        if self.curr_editor_idx >= self.editors.len() {
            if self.curr_editor_idx > 0 {
                error!("current editor points further than number opened editors!");
                return subwidget!(Self.tree_widget);
            }

            subwidget!(Self.no_editor)
        } else {
            let idx1 = self.curr_editor_idx;
            let idx2 = self.curr_editor_idx;
            SubwidgetPointer::new(
                Box::new(move |s: &Self| { s.editors.get(idx1).map(|w| w.as_any()).unwrap_or(&s.no_editor) }),
                Box::new(move |s: &mut Self| { s.editors.get_mut(idx2).map(|w| w as &mut dyn Widget).unwrap_or(&mut s.no_editor) }),
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

    pub fn set_search_result(&mut self, crv_op: Option<CodeResultsView>) {
        self.crv_op = crv_op;
    }
}

impl Widget for MainView {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "main_view"
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
                if self.editors.is_empty() {
                    debug!("ignoring browse_buffers request - no editors open.");
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
                    if *pos >= self.editors.len() {
                        error!("received FuzzyBufferHit for an index {} and len is {}, ignoring", pos, self.editors.len());
                    } else {
                        self.curr_editor_idx = *pos;
                    }
                    // removing the dialog
                    self.hover = None;

                    None
                }
                MainViewMsg::FindReferences { ref mut promise_op } => {
                    if let Some(mut promise) = promise_op.take() {
                        promise.update();
                        if promise.state() != PromiseState::Broken {
                            self.crv_op = Some(CodeResultsView::new(self.providers.config().clone(),
                                                                    self.providers.tree_sitter().clone(),
                                                                    self.providers.fsf().clone(),
                                                                    self.providers.clipboard().clone(),
                                                                    "TODO bla bla bla".to_string(),
                                                                    //TODO
                                                                    Box::new(promise)));
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
    fn get_layout(&self, sc: SizeConstraint) -> Box<dyn Layout<Self>> {
        let left_column = LeafLayout::new(subwidget!(Self.tree_widget)).boxed();
        let right_column = if self.crv_op.is_none() {
            LeafLayout::new(self.get_curr_editor_ptr()).boxed()
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


        //TODO(subwidgetpointermap)
        let max_size = sc.as_finite().unwrap_or(self.size());

        let res = if let Some(hover) = &self.hover {
            match hover {
                HoverItem::FuzzySearch(_fuzzy) => {
                    let rect = MainView::get_hover_rect(max_size);

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
                        rect,
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
        self.get_curr_editor_ptr()
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