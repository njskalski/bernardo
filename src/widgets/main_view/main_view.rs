use std::path::PathBuf;

use crate::{AnyMsg, InputEvent, LocalFilesystemProvider, Output, SizeConstraint, Theme, Widget};
use crate::io::filesystem_tree::filesystem_provider::FilesystemProvider;
use crate::primitives::xy::XY;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::tree_view::tree_view::TreeViewWidget;

const MIN_VIEW_SIZE: XY = XY::new(32, 10);

pub struct MainView {
    wid: WID,
    fs: Box<dyn FilesystemProvider>,
    tree: TreeViewWidget<PathBuf>,
}

impl MainView {
    pub fn new(root_dir: PathBuf) -> MainView {
        let local = LocalFilesystemProvider::new(root_dir);
        let tree = TreeViewWidget::new(local.get_root());

        MainView {
            wid: get_new_widget_id(),
            fs: Box::new(local),
            tree,
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
        todo!()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        todo!()
    }
}