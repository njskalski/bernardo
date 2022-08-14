use std::path::PathBuf;
use std::rc::Rc;

use log::{debug, error, warn};

use crate::{AnyMsg, ConfigRef, InputEvent, Output, SizeConstraint, Widget};
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::path::SPath;
use crate::io::sub_output::SubOutput;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::xy::XY;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::w7e::navcomp_group::NavCompGroupRef;
use crate::widget::any_msg::AsAny;
use crate::widget::complex_widget::ComplexWidget;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::fuzzy_search::fsf_provider::{FsfProvider, SPathMsg};
use crate::widgets::fuzzy_search::fuzzy_search::{DrawComment, FuzzySearchWidget};
use crate::widgets::main_view::display_state::{Focus, MainViewDisplayState};
use crate::widgets::main_view::editor_group::EditorGroup;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::spath_tree_view_node::FileTreeNode;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;
use crate::widgets::with_scroll::WithScroll;

const MIN_VIEW_SIZE: XY = XY::new(32, 10);

pub enum HoverItem {
    FuzzySearch(FuzzySearchWidget),
}

pub struct MainView {
    wid: WID,
    /*
    I use a simplified "display state" model, not the GenericFocusGroup approach, to see how much effort the Generic one saves.
    caveat: whenever focusing on editor, make sure to set curr_editor_index as well. It's a temporary solution, so I don't wrap it.
     */
    display_state: MainViewDisplayState,

    // TODO PathBuf -> WrappedRcPath? See profiler.
    tree_widget: WithScroll<TreeViewWidget<SPath, FileTreeNode>>,

    editors: EditorGroup,
    no_editor: NoEditorWidget,

    // Providers
    tree_sitter: Rc<TreeSitterWrapper>,
    fsf: FsfRef,
    clipboard: ClipboardRef,
    config: ConfigRef,

    hover: Option<HoverItem>,
}

impl MainView {
    pub fn new(config: ConfigRef,
               tree_sitter: Rc<TreeSitterWrapper>,
               fsf: FsfRef,
               clipboard: ClipboardRef,
               nav_comp_group: NavCompGroupRef,
    ) -> MainView {
        let root = fsf.root();
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
            display_state: MainViewDisplayState::default(),
            tree_widget: WithScroll::new(tree, ScrollDirection::Vertical),
            editors: EditorGroup::new(
                config.clone(),
                nav_comp_group.clone(),
            ),
            no_editor: NoEditorWidget::default(),
            tree_sitter,
            fsf,
            clipboard,
            config,
            hover: None,
        }
    }

    pub fn with_empty_editor(mut self) -> Self {
        self.open_empty_editor_and_focus();
        self
    }

    fn open_empty_editor_and_focus(&mut self) {
        let idx = self.editors.open_empty(self.tree_sitter.clone(), self.fsf.clone(), self.clipboard.clone());
        self.display_state.focus = Focus::Editor;
        self.display_state.curr_editor_idx = Some(idx);
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

    fn internal_layout(&mut self, max_size: XY) -> Vec<WidgetIdRect> {
        let tree_widget = &mut self.tree_widget;

        let mut left_column = LeafLayout::new(tree_widget);
        let editor_or_not = match self.display_state.curr_editor_idx {
            None => &mut self.no_editor as &mut dyn Widget,
            Some(idx) => self.editors.get_mut(idx).map(|w| w as &mut dyn Widget).unwrap_or(&mut self.no_editor),
        };

        let mut right_column = LeafLayout::new(editor_or_not);

        let mut bg_layout = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0),
                  &mut left_column)
            .with(SplitRule::Proportional(4.0),
                  &mut right_column,
            );


        let res = if let Some(hover) = &mut self.hover {
            match hover {
                HoverItem::FuzzySearch(fuzzy) => {
                    let rect = MainView::get_hover_rect(max_size);
                    HoverLayout::new(
                        &mut bg_layout,
                        &mut LeafLayout::new(fuzzy),
                        rect,
                    ).calc_sizes(max_size)
                }
            }
        } else {
            bg_layout.calc_sizes(max_size)
        };

        res
    }

    pub fn open_file(&mut self, ff: SPath) -> bool {
        debug!("opening file {:?}", ff);

        if let Some(idx) = self.editors.get_if_open(&ff) {
            self.display_state.focus = Focus::Editor;
            self.display_state.curr_editor_idx = Some(idx);
            true
        } else {
            self.editors.open_file(self.tree_sitter.clone(), ff, self.clipboard.clone()).map(|idx| {
                self.display_state.focus = Focus::Editor;
                self.display_state.curr_editor_idx = Some(idx);
            }).is_ok()
        }
    }

    fn open_fuzzy_search_in_files(&mut self) {
        self.hover = Some(
            HoverItem::FuzzySearch(FuzzySearchWidget::new(
                |_| Some(Box::new(MainViewMsg::ClozeHover))
            ).with_provider(
                Box::new(FsfProvider::new(self.fsf.clone()).with_ignores_filter())
            ).with_draw_comment_setting(DrawComment::Highlighted))
        );
    }

    fn open_fuzzy_buffer_list(&mut self) {
        self.hover = Some(
            HoverItem::FuzzySearch(FuzzySearchWidget::new(
                |_| Some(Box::new(MainViewMsg::ClozeHover))
            ).with_provider(
                self.editors.get_buffer_list_provider()
            ).with_draw_comment_setting(DrawComment::Highlighted))
        );
    }
}

impl Widget for MainView {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "main_view"
    }

    fn min_size(&self) -> XY {
        MIN_VIEW_SIZE
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        let max_size = sc.visible_hint().size;
        let res_sizes = self.internal_layout(max_size);
        self.display_state.wirs = res_sizes;
        max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        // debug!("main_view.on_input {:?}", input_event);

        return match input_event {
            InputEvent::FocusUpdate(focus_update) if self.display_state.will_accept_update_focus(focus_update) => {
                MainViewMsg::FocusUpdateMsg(focus_update).someboxed()
            }
            InputEvent::KeyInput(key) if key == self.config.keyboard_config.global.new_buffer => {
                MainViewMsg::OpenNewFile.someboxed()
            }
            InputEvent::KeyInput(key) if key == self.config.keyboard_config.global.fuzzy_file => {
                MainViewMsg::OpenFuzzyFiles.someboxed()
            }
            InputEvent::KeyInput(key) if key == self.config.keyboard_config.global.browse_buffers => {
                if self.editors.is_empty() {
                    debug!("ignoring browse_buffers request - no editors open.");
                    None
                } else {
                    MainViewMsg::OpenFuzzyBuffers.someboxed()
                }
            }
            _ => None
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        // debug!("main_view.update {:?}", msg);

        if let Some(main_view_msg) = msg.as_msg::<MainViewMsg>() {
            return match main_view_msg {
                MainViewMsg::FocusUpdateMsg(focus_update) => {
                    if !self.display_state.update_focus(*focus_update) {
                        error!("failed to accept focus update")
                    }

                    None
                }
                MainViewMsg::TreeExpandedFlip { expanded, item } => {
                    None
                }
                MainViewMsg::TreeSelected { item } => {
                    if !self.open_file(item.clone()) {
                        error!("failed open_file");
                    }

                    None
                }
                MainViewMsg::OpenFuzzyFiles => {
                    self.open_fuzzy_search_in_files();
                    None
                }
                MainViewMsg::ClozeHover => {
                    if self.hover.is_none() {
                        error!("expected self.hover to be not None.");
                    }

                    self.hover = None;
                    None
                }
                MainViewMsg::OpenFuzzyBuffers => {
                    self.open_fuzzy_buffer_list();
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
                        self.display_state.curr_editor_idx = Some(*pos);
                    }
                    // removing the dialog
                    self.hover = None;

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
        // that's not how I would write it, but borrow checker does not appreciate my style.
        if self.hover.is_some() {
            return match self.hover.as_ref().unwrap() {
                HoverItem::FuzzySearch(fuzzy) => Some(fuzzy),
            };
        } else {
            match self.display_state.focus {
                Focus::Tree => Some(&self.tree_widget as &dyn Widget),
                Focus::Editor => {
                    self.display_state.curr_editor_idx.map(|idx| {
                        self.editors.get(idx).map(|w| w as &dyn Widget).unwrap_or(
                            &self.no_editor
                        )
                    })
                }
            }
        }
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        if self.hover.is_some() {
            return match self.hover.as_mut().unwrap() {
                HoverItem::FuzzySearch(fuzzy) => Some(fuzzy),
            };
        } else {
            match self.display_state.focus {
                Focus::Tree => Some(&mut self.tree_widget as &mut dyn Widget),
                Focus::Editor => {
                    Some(match self.display_state.curr_editor_idx {
                        None => &mut self.no_editor as &mut dyn Widget,
                        Some(idx) => self.editors.get_mut(idx).map(|w| w as &mut dyn Widget).unwrap_or(
                            &mut self.no_editor
                        )
                    })
                }
            }
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        ComplexWidget::render(self, theme, focused, output)
    }
}

impl ComplexWidget for MainView {
    fn subwidgets_mut(&mut self) -> Box<dyn Iterator<Item=&mut dyn Widget> + '_> where Self: Sized {
        let mut items = vec![&mut self.tree_widget as &mut dyn Widget];

        let editor_or_not = match self.display_state.curr_editor_idx {
            None => &mut self.no_editor as &mut dyn Widget,
            Some(idx) => self.editors.get_mut(idx).map(|w| w as &mut dyn Widget).unwrap_or(&mut self.no_editor),
        };
        items.push(editor_or_not);

        if self.hover.is_some() {
            match self.hover.as_mut().unwrap() {
                HoverItem::FuzzySearch(fuzzy) => {
                    items.push(fuzzy);
                }
            }
        };

        Box::new(items.into_iter())
    }

    fn subwidgets(&self) -> Box<dyn Iterator<Item=&dyn Widget> + '_> where Self: Sized {
        let mut items = vec![&self.tree_widget as &dyn Widget];

        let editor_or_not = match self.display_state.curr_editor_idx {
            None => &self.no_editor as &dyn Widget,
            Some(idx) => self.editors.get(idx).map(|w| w as &dyn Widget).unwrap_or(&self.no_editor),
        };
        items.push(editor_or_not);

        if self.hover.is_some() {
            match self.hover.as_ref().unwrap() {
                HoverItem::FuzzySearch(fuzzy) => {
                    items.push(fuzzy);
                }
            }
        };

        Box::new(items.into_iter())
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        // that's not how I would write it, but borrow checker does not appreciate my style.
        if self.hover.is_some() {
            return match self.hover.as_ref().unwrap() {
                HoverItem::FuzzySearch(fuzzy) => Some(fuzzy),
            };
        } else {
            match self.display_state.focus {
                Focus::Tree => Some(&self.tree_widget as &dyn Widget),
                Focus::Editor => {
                    self.display_state.curr_editor_idx.map(|idx| {
                        self.editors.get(idx).map(|w| w as &dyn Widget).unwrap_or(
                            &self.no_editor
                        )
                    })
                }
            }
        }
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        if self.hover.is_some() {
            return match self.hover.as_mut().unwrap() {
                HoverItem::FuzzySearch(fuzzy) => Some(fuzzy),
            };
        } else {
            match self.display_state.focus {
                Focus::Tree => Some(&mut self.tree_widget as &mut dyn Widget),
                Focus::Editor => {
                    Some(match self.display_state.curr_editor_idx {
                        None => &mut self.no_editor as &mut dyn Widget,
                        Some(idx) => self.editors.get_mut(idx).map(|w| w as &mut dyn Widget).unwrap_or(
                            &mut self.no_editor
                        )
                    })
                }
            }
        }
    }
}