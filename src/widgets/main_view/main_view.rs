use std::borrow::Borrow;
use std::path::PathBuf;

use log::{debug, warn};

use crate::{AnyMsg, InputEvent, LocalFilesystemProvider, Output, Theme, Widget};
use crate::experiments::scroll::Scroll;
use crate::io::filesystem_tree::filesystem_provider::FilesystemProvider;
use crate::io::sub_output::SubOutput;
use crate::layout::cached_sizes::DisplayState;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::xy::XY;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::editor_view::text_editor::TextEditorWidget;
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::tree_view::tree_view::TreeViewWidget;

pub struct MainView {
    wid: WID,
    tree_widget: TreeViewWidget<PathBuf>,
    tree_scroll: Scroll,

    no_editor_widget: NoEditorWidget,

    local_filesystem: Box<dyn FilesystemProvider>,

    display_state: Option<DisplayState>,
}

impl MainView {
    pub fn new(root_dir: PathBuf) -> Self {
        let fs = LocalFilesystemProvider::new(root_dir);

        let tree = TreeViewWidget::<PathBuf>::new(fs.get_root());
        MainView {
            wid: get_new_widget_id(),
            tree_widget: tree,
            tree_scroll: Scroll::new(XY::new(100, 1000)),//TODO

            no_editor_widget: NoEditorWidget::new(),
            local_filesystem: Box::new(fs),

            display_state: None,
        }
    }

    fn todo_internal_layout(&mut self, max_size: XY) -> Vec<WidgetIdRect> {
        let tree_widget = &mut self.tree_widget;
        let no_editor_widget = &mut self.no_editor_widget;

        let mut left_column = LeafLayout::new(tree_widget);
        let mut editor_pane = LeafLayout::new(no_editor_widget);

        let mut layout = SplitLayout::new(SplitDirection::Horizontal)
            .with(SplitRule::Proportional(1.0),
                  &mut left_column)
            .with(SplitRule::Proportional(4.0),
                  &mut editor_pane,
            );

        let res = layout.calc_sizes(max_size);

        res
    }

    fn todo_wid_to_widget_or_self(&self, wid: WID) -> &dyn Widget {
        if self.no_editor_widget.id() == wid {
            return &self.no_editor_widget
        }
        if self.tree_widget.id() == wid {
            return &self.tree_widget
        }

        warn!("todo_wid_to_widget_or_self on {} failed - widget {} not found. Returning self.", wid, self.id());

        self
    }

    fn todo_wid_to_widget_or_self_mut(&mut self, wid: WID) -> &mut dyn Widget {
        if self.no_editor_widget.id() == wid {
            return &mut self.no_editor_widget
        }
        if self.tree_widget.id() == wid {
            return &mut self.tree_widget
        }

        warn!("todo_wid_to_widget_mut_or_self on {} failed - widget {} not found. Returning self.", wid, self.id());

        self
    }
}

impl Widget for MainView {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "main_window"
    }

    fn min_size(&self) -> XY {
        XY::new(30, 20) // completely arbitrary
    }

    fn layout(&mut self, max_size: XY) -> XY {
        let res_sizes = self.todo_internal_layout(max_size);

        for wir in &res_sizes {
            if wir.wid == self.tree_widget.id() {
                self.tree_scroll.follow_anchor(wir.rect.size, self.tree_widget.anchor());
            }
        }

        let focus_op = self.display_state.as_ref().map(|ds| ds.focus_group.get_focused());

        self.display_state = Some(DisplayState::new(max_size, res_sizes));

        // re-setting focus.
        match (focus_op, &mut self.display_state) {
            (Some(focus), Some(ds)) => { ds.focus_group.set_focused(focus); },
            _ => {}
        };

        max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn get_focused(&self) -> &dyn Widget {
        return match self.display_state.borrow().as_ref() {
            Some(cached_sizes) => {
                let focused_wid = cached_sizes.focus_group.get_focused();
                self.todo_wid_to_widget_or_self(focused_wid)
            }
            None => {
                warn!("get_focused on {} failed - no cached_sizes. Returning self.", self.id());
                self
            }
        }
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        let wid_op: Option<WID> = match &self.display_state {
            Some(cached_sizes) => {
                Some(cached_sizes.focus_group.get_focused())
            }
            None => {
                warn!("get_focused on {} failed - no cached_sizes.", self.id());
                None
            }
        };

        return match wid_op {
            None => self,
            Some(wid) => self.todo_wid_to_widget_or_self_mut(wid)
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        match self.display_state.borrow().as_ref() {
            None => warn!("failed rendering main_view without cached_sizes"),
            Some(display_state) => {
                debug!("widget_sizes : {:?}", display_state.widget_sizes);
                for wir in &display_state.widget_sizes {
                    let widget = self.todo_wid_to_widget_or_self(wir.wid);

                    if widget.id() == self.id() {
                        warn!("render: failed to match WID {} to sub-widget in save_file_dialog {}", wir.wid, self.id());
                        continue;
                    }

                    let mut sub_output = &mut SubOutput::new(Box::new(output), wir.rect);

                    if widget.id() != self.tree_widget.id() {
                        widget.render(theme,
                                      display_state.focus_group.get_focused() == widget.id(),
                                      sub_output);
                    }

                    if widget.id() == self.tree_widget.id() {
                        self.tree_scroll.render_within(sub_output,
                                                       &self.tree_widget,
                                                       theme,
                                                       display_state.focus_group.get_focused() == widget.id(),
                        );
                    }
                }
            }
        }
    }
}