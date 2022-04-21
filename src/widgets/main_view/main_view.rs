use std::borrow::Borrow;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use log::{debug, error, warn};
use syntect::html::IncludeBackground::No;

use crate::{AnyMsg, InputEvent, Keycode, Output, SizeConstraint, Widget};
use crate::experiments::filename_to_language::filename_to_language;
use crate::fs::file_front::FileFront;
use crate::fs::fsfref::FsfRef;
use crate::io::sub_output::SubOutput;
use crate::layout::display_state::DisplayState;
use crate::layout::dummy_layout::DummyLayout;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::theme::Theme;
use crate::primitives::xy::XY;
use crate::text::buffer_state::BufferState;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::editor_view::msg::EditorViewMsg;
use crate::widgets::fuzzy_search::fsf_provider::{FileFrontMsg, FsfProvider};
use crate::widgets::fuzzy_search::fuzzy_search::{DrawComment, FuzzySearchWidget};
use crate::widgets::main_view::editor_group::EditorGroup;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;
use crate::widgets::with_scroll::WithScroll;

const MIN_VIEW_SIZE: XY = XY::new(32, 10);

pub enum HoverItem {
    FuzzySearch(FuzzySearchWidget),
}

pub struct MainView {
    wid: WID,
    display_state: Option<DisplayState>,

    // TODO PathBuf -> WrappedRcPath? See profiler.
    tree_widget: WithScroll<TreeViewWidget<PathBuf, FileFront>>,

    editors: EditorGroup,

    tree_sitter: Rc<TreeSitterWrapper>,

    fsf: FsfRef,

    hover: Option<HoverItem>,
}

impl MainView {
    pub fn new(tree_sitter: Rc<TreeSitterWrapper>, fs: FsfRef) -> MainView {
        let root_node = fs.get_root();
        let tree = TreeViewWidget::new(root_node)
            .with_on_flip_expand(|widget| {
                let (_, item) = widget.get_highlighted();

                Some(Box::new(MainViewMsg::TreeExpandedFlip {
                    expanded: widget.is_expanded(item.id()),
                    item,
                }))
            })
            .with_on_select_hightlighted(|widget| {
                let (_, item) = widget.get_highlighted();
                Some(Box::new(MainViewMsg::TreeSelected { item }))
            });

        MainView {
            wid: get_new_widget_id(),
            display_state: None,
            tree_widget: WithScroll::new(tree, ScrollDirection::Vertical),
            editors: EditorGroup::default(),
            tree_sitter,
            fsf: fs,
            hover: None,
        }
    }

    pub fn with_empty_editor(mut self) -> Self {
        self.editors.open_empty(self.tree_sitter.clone(), self.fsf.clone(), true);
        self
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
        let mut right_column = LeafLayout::new(self.editors.curr_editor_mut());

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

    fn open_file(&mut self, ff: FileFront) -> bool {
        debug!("opening file {:?}", ff.path());
        // TODO show error?
        self.editors.open_file(self.tree_sitter.clone(), ff, true).is_ok()
    }

    fn open_fuzzy_search_in_files(&mut self) {
        self.hover = Some(
            HoverItem::FuzzySearch(FuzzySearchWidget::new(
                |_| Some(Box::new(MainViewMsg::FuzzyClose))
            ).with_provider(
                Box::new(FsfProvider::new(self.fsf.clone()).with_ignores_filter())
            ).with_draw_comment_setting(DrawComment::Highlighted))
        );
    }

    // TODO
    // fn set_focus_on_editor(&mut self) {
    //     if let Some(wid) = self.curr_editor().map(|w| w.id()) {
    //         if let Some(ds) = &mut self.display_state {
    //             if ds.focus_group.set_focused(wid) {
    //                 error!("failed to update focus to {}", wid);
    //             }
    //         }
    //     } else {
    //         if let Some(ds) = &mut self.display_state {
    //             if ds.focus_group.set_focused(self.no_editor.id()) {
    //                 error!("failed to update focus to no_editor {}", self.no_editor.id());
    //             }
    //         }
    //     }
    // }
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
        let max_size = sc.hint().size;

        // TODO this lazy relayouting kills resizing on data change.
        // TODO relayouting destroys focus selection.

        let res_sizes = self.internal_layout(max_size);

        // debug!("size {}, res_sizes {:?}", max_size, res_sizes);

        // Retention of focus. Not sure if it should be here.
        let focus_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());

        self.display_state = Some(DisplayState::new(max_size, res_sizes));

        // re-setting focus.
        match (focus_op, &mut self.display_state) {
            (Some(focus), Some(ds)) => { ds.focus_group.set_focused(focus); }
            _ => {}
        };

        max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug!("main_view.on_input {:?}", input_event);

        return match input_event {
            InputEvent::FocusUpdate(focus_update) => {
                Some(Box::new(MainViewMsg::FocusUpdateMsg(focus_update)))
            }
            InputEvent::KeyInput(key) if key.modifiers.ctrl && key.keycode == Keycode::Char('h') => {
                Some(Box::new(MainViewMsg::OpenFuzzyFiles))
            }
            _ => None
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("main_view.update {:?}", msg);

        if let Some(main_view_msg) = msg.as_msg::<MainViewMsg>() {
            return match main_view_msg {
                MainViewMsg::FocusUpdateMsg(focus_update) => {
                    warn!("updating focus");
                    let fc = *focus_update;
                    let ds: &mut DisplayState = self.display_state.as_mut().unwrap();
                    let fg = &mut ds.focus_group;
                    let msg = fg.update_focus(fc);
                    warn!("focus updated {}", msg);
                    None
                }
                MainViewMsg::TreeExpandedFlip { expanded, item } => {
                    None
                }
                MainViewMsg::TreeSelected { item } => {
                    if self.open_file(item.clone()) {
                        // self.set_focus_on_editor();
                    } else {
                        error!("failed open_file");
                    }

                    None
                }
                MainViewMsg::OpenFuzzyFiles => {
                    self.open_fuzzy_search_in_files();
                    None
                }
                MainViewMsg::FuzzyClose => {
                    if self.hover.is_none() {
                        error!("expected self.hover to be not None.");
                    }

                    self.hover = None;
                    None
                }
            };
        };

        if let Some(fuzzy_file_msg) = msg.as_msg::<FileFrontMsg>() {
            return match fuzzy_file_msg {
                FileFrontMsg::Hit(file_front) => {
                    self.open_file(file_front.clone());
                    self.hover = None;
                    // self.set_focus_on_editor();
                    None
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
            let wid_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());
            wid_op.map(move |wid| self.get_subwidget(wid)).flatten()
        }
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        if self.hover.is_some() {
            return match self.hover.as_mut().unwrap() {
                HoverItem::FuzzySearch(fuzzy) => Some(fuzzy),
            };
        } else {
            let wid_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());
            wid_op.map(move |wid| self.get_subwidget_mut(wid)).flatten()
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let focused_id_op = self.get_focused().map(|f| f.id());
        match self.display_state.borrow().as_ref() {
            None => {
                warn!("failed rendering main_view without cached_sizes");
                return;
            }
            Some(cached_sizes) => {
                // debug!("widget_sizes : {:?}", cached_sizes.widget_sizes);
                for wir in &cached_sizes.widget_sizes {
                    match self.get_subwidget(wir.wid) {
                        Some(widget) => {
                            let sub_output = &mut SubOutput::new(output, wir.rect);
                            widget.render(theme,
                                          Some(widget.id()) == focused_id_op,
                                          sub_output,
                            );
                        }
                        None => {
                            warn!("subwidget {} not found!", wir.wid);
                        }
                    }
                }
            }
        }
    }

    fn subwidgets_mut(&mut self) -> Box<dyn Iterator<Item=&mut dyn Widget> + '_> where Self: Sized {
        let mut items = vec![&mut self.tree_widget as &mut dyn Widget, self.editors.curr_editor_mut()];

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
        let mut items = vec![&self.tree_widget as &dyn Widget, self.editors.curr_editor()];

        if self.hover.is_some() {
            match self.hover.as_ref().unwrap() {
                HoverItem::FuzzySearch(fuzzy) => {
                    items.push(fuzzy);
                }
            }
        };

        Box::new(items.into_iter())
    }
}