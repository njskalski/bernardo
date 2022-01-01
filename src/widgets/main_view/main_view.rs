use std::path::PathBuf;

use crate::{AnyMsg, InputEvent, LocalFilesystemProvider, Output, Theme, Widget};
use crate::experiments::scroll::Scroll;
use crate::io::filesystem_tree::filesystem_provider::FilesystemProvider;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::xy::XY;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::editor_view::text_editor::TextEditorWidget;
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::tree_view::tree_view::TreeViewWidget;

struct MainView {
    wid: WID,
    tree_widget: TreeViewWidget<PathBuf>,
    tree_scroll: Scroll,

    no_editor_widget: NoEditorWidget,

    local_filesystem: Box<dyn FilesystemProvider>,
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
        }
    }

    fn todo_internal_layout(&mut self, max_size: XY) -> Vec<WidgetIdRect> {
        let tree_widget = &mut self.tree_widget;
        let no_editor_widget = &mut self.no_editor_widget;

        let mut left_column = LeafLayout::new(tree_widget);
        let mut editor_pane = LeafLayout::new(no_editor_widget);

        let mut layout = SplitLayout::new(SplitDirection::Vertical)
            .with(SplitRule::Proportional(1.0),
                  &mut left_column)
            .with(SplitRule::Proportional(4.0),
                  &mut editor_pane,
            );

        let res = layout.calc_sizes(max_size);

        res
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

        max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn get_focused(&self) -> &dyn Widget {
        todo!()
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        todo!()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        todo!()
    }
}