/*
this widget is supposed to offer:
- tree view on the right, along with scrolling,
- file list view on the most of display (with scrolling as well)
- filename edit box
- buttons save and cancel

I hope I will discover most of functional constraints while implementing it.
 */

use crate::widget::any_msg::AnyMsg;
use std::fmt::{Debug, Formatter};
use crate::layout::layout::Layout;
use crate::widget::stupid_tree::StupidTree;
use crate::widget::tree_view::TreeViewWidget;

pub struct SaveFileDialogWidget {
    layout: Box<Layout<SaveFileDialogWidget>>,
    mock_file_tree: StupidTree,
    tree_widget: TreeViewWidget<StupidTree>,

}

#[derive(Clone, Copy, Debug)]
pub enum SaveFileDialogMsg {}

impl AnyMsg for SaveFileDialogMsg {}

impl SaveFileDialogWidget {}

