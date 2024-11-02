use crate::fs::path::SPath;
use crate::widgets::context_menu::widget::ContextMenuWidget;
use crate::widgets::spath_tree_view_node::FileTreeNode;

pub type NewFuzzyFileSearch = ContextMenuWidget<SPath, FileTreeNode>;
