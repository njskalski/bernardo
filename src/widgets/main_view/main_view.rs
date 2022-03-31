use std::borrow::Borrow;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use log::{debug, error, warn};

use crate::{AnyMsg, InputEvent, Output, SizeConstraint, Widget};
use crate::experiments::filename_to_language::filename_to_language;
use crate::io::filesystem_tree::file_front::FileFront;
use crate::io::filesystem_tree::fsfref::FsfRef;
use crate::io::sub_output::SubOutput;
use crate::layout::display_state::DisplayState;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::theme::Theme;
use crate::primitives::xy::XY;
use crate::text::buffer_state::BufferState;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::tree_view::tree_view::TreeViewWidget;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;
use crate::widgets::with_scroll::WithScroll;

const MIN_VIEW_SIZE: XY = XY::new(32, 10);

pub struct MainView {
    wid: WID,
    display_state: Option<DisplayState>,

    tree_widget: WithScroll<TreeViewWidget<PathBuf, Rc<FileFront>>>,
    no_editor: NoEditorWidget,
    editor: Option<WithScroll<EditorView>>,
    tree_sitter: Rc<TreeSitterWrapper>,

    fs: FsfRef,
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
            no_editor: NoEditorWidget::new(),
            editor: None,
            tree_sitter,
            fs,
        }
    }

    pub fn with_empty_editor(self) -> Self {
        MainView {
            editor: Some(
                WithScroll::new(
                    EditorView::new(self.tree_sitter.clone(), self.fs.clone()),
                    ScrollDirection::Both,
                )
            ),
            ..self
        }
    }

    fn internal_layout(&mut self, max_size: XY) -> Vec<WidgetIdRect> {
        let tree_widget = &mut self.tree_widget;
        let _no_editor = &mut self.no_editor;

        let mut left_column = LeafLayout::new(tree_widget);
        let mut right_column = self.editor.as_mut()
            .map(|editor| LeafLayout::new(editor))
            .unwrap_or(LeafLayout::new(&mut self.no_editor));

        let mut layout = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0),
                  &mut left_column)
            .with(SplitRule::Proportional(4.0),
                  &mut right_column,
            );

        let res = layout.calc_sizes(max_size);

        res
    }

    fn open_file(&mut self, path: &Path) -> bool {
        debug!("opening file {:?}", path);

        // TODO this maybe needs to be moved to other place, but there is no other place yet.

        let lang_id = filename_to_language(path);

        match self.fs.todo_read_file(path) {
            Ok(rope) => {
                self.editor = Some(
                    WithScroll::new(
                        EditorView::new(self.tree_sitter.clone(), self.fs.clone())
                            .with_buffer(
                                BufferState::new(self.tree_sitter.clone())
                                    .with_text_from_rope(rope, lang_id)
                            ).with_path_op(
                            path.parent().map(|p| p.to_owned())
                        ),
                        ScrollDirection::Both,
                    ));
                true
            }
            Err(_) => {
                error!("failed to load file {:?}", path);
                false
            }
        }
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
            _ => None
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("main_view.update {:?}", msg);

        let our_msg = msg.as_msg::<MainViewMsg>();
        if our_msg.is_none() {
            warn!("expecetd MainViewMsg, got {:?}", msg);
            return None;
        }

        return match our_msg.unwrap() {
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
                if *expanded {
                    self.fs.todo_expand(item.id());
                }
                None
            }
            MainViewMsg::TreeSelected { item } => {
                if self.open_file(item.id().as_path()) {
                    if let (Some(ds), Some(editor)) = (&mut self.display_state, &self.editor) {
                        if ds.focus_group.set_focused(editor.id()) {
                            error!("failed to update focus after update file");
                        }
                    }
                } else {
                    error!("failed open_file");
                }

                None
            }
            unknown_msg => {
                warn!("SaveFileDialog.update : unknown message {:?}", unknown_msg);
                None
            }
        };
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        let wid_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());
        wid_op.map(|wid| self.get_subwidget(wid)).flatten()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        let wid_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());
        wid_op.map(move |wid| self.get_subwidget_mut(wid)).flatten()
    }

    fn render(&self, theme: &Theme, _focused: bool, output: &mut dyn Output) {
        match self.display_state.borrow().as_ref() {
            None => warn!("failed rendering main_view without cached_sizes"),
            Some(cached_sizes) => {
                // debug!("widget_sizes : {:?}", cached_sizes.widget_sizes);
                for wir in &cached_sizes.widget_sizes {
                    match self.get_subwidget(wir.wid) {
                        Some(widget) => {
                            let sub_output = &mut SubOutput::new(output, wir.rect);
                            widget.render(theme,
                                          cached_sizes.focus_group.get_focused() == widget.id(),
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
        match &mut self.editor {
            None => Box::new(vec![&mut self.tree_widget as &mut dyn Widget, &mut self.no_editor].into_iter()),
            Some(editor) => Box::new(vec![&mut self.tree_widget as &mut dyn Widget, editor].into_iter())
        }
    }

    fn subwidgets(&self) -> Box<dyn Iterator<Item=&dyn Widget> + '_> where Self: Sized {
        match &self.editor {
            None => Box::new(vec![&self.tree_widget as &dyn Widget, &self.no_editor].into_iter()),
            Some(editor) => Box::new(vec![&self.tree_widget as &dyn Widget, editor].into_iter())
        }
    }
}