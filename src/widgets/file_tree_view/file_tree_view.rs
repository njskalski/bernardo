/*
This is a piece of specialized code for TreeView of SPath
 */

use std::borrow::Cow;

use log::{debug, error};
use streaming_iterator::StreamingIterator;

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::fs::path::SPath;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::tree::filter_policy::FilterPolicy;
use crate::primitives::tree::tree_node::{ClosureFilter, FilterRef, TreeItFilter, TreeNode};
use crate::primitives::xy::XY;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::context_bar_item::ContextBarItem;
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::file_tree_view::msg::FileTreeViewMsg;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::widgets::spath_tree_view_node::FileTreeNode;
use crate::widgets::tree_view::tree_view::{LabelHighlighter, TreeViewWidget};
use crate::widgets::with_scroll::with_scroll::WithScroll;

pub struct FileTreeViewWidget {
    id: WID,
    config: ConfigRef,
    tree_view_widget: WithScroll<TreeViewWidget<SPath, FileTreeNode>>,
}

impl FileTreeViewWidget {
    const TYPENAME: &'static str = "file_tree_view_widget";

    pub fn new(config: ConfigRef, root: SPath) -> FileTreeViewWidget {
        let tree = TreeViewWidget::new(FileTreeNode::new(root.clone()))
            .with_on_flip_expand(Box::new(|widget| {
                let (_, item) = widget.get_highlighted();

                Some(Box::new(MainViewMsg::TreeExpandedFlip {
                    expanded: widget.is_expanded(item.id()),
                    item: item.spath().clone(),
                }))
            }))
            .with_on_hit(Box::new(|widget| {
                let (_, item) = widget.get_highlighted();
                Some(Box::new(MainViewMsg::TreeSelected {
                    item: item.spath().clone(),
                }))
            }));

        FileTreeViewWidget {
            id: get_new_widget_id(),
            config,
            tree_view_widget: WithScroll::new(ScrollDirection::Both, tree),
        }
    }

    pub fn get_hidden_files_filter() -> FilterRef<FileTreeNode> {
        ClosureFilter::new(|node: &FileTreeNode| -> bool {
            if let Some(filename) = node.spath().last_file_name() {
                if let Some(filename) = filename.to_str() {
                    if filename.starts_with(&['.']) {
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            } else {
                true
            }
        })
        .arc_box()
    }

    pub fn set_hidden_files_filter(&mut self, enabled: bool) {
        if enabled {
            self.tree_view_widget
                .internal_mut()
                .set_filter_op(Some(Self::get_hidden_files_filter()), FilterPolicy::MatchNode);
        } else {
            self.tree_view_widget.internal_mut().set_filter_op(None, FilterPolicy::MatchNode);
        }
    }

    pub fn with_hidden_files_filter(mut self, enabled: bool) -> Self {
        self.set_hidden_files_filter(enabled);
        self
    }

    pub fn are_hidden_files_filtered(&self) -> bool {
        self.tree_view_widget.internal().is_filter_set()
    }

    pub fn toggle_hidden_files_filter(&mut self) {
        let is_filter_enabled = self.are_hidden_files_filtered();
        self.set_hidden_files_filter(!is_filter_enabled);
    }
}

impl Widget for FileTreeViewWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn desc(&self) -> String {
        self.tree_view_widget.desc()
    }

    fn full_size(&self) -> XY {
        self.tree_view_widget.full_size()
    }

    fn size_policy(&self) -> SizePolicy {
        self.tree_view_widget.size_policy()
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.tree_view_widget.layout(screenspace)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        match input_event {
            InputEvent::KeyInput(key) if key == self.config.keyboard_config.file_tree.toggle_hidden_files => {
                FileTreeViewMsg::ToggleHiddenFilesFilter.someboxed()
            }
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("file_tree_view.update {:?}", msg);

        if let Some(file_tree_view_msg) = msg.as_msg::<FileTreeViewMsg>() {
            match file_tree_view_msg {
                FileTreeViewMsg::ToggleHiddenFilesFilter => {
                    self.toggle_hidden_files_filter();
                    None
                }
            }
        } else {
            Some(msg)
        }
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        Some(&self.tree_view_widget)
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        Some(&mut self.tree_view_widget)
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        self.tree_view_widget.render(theme, focused, output)
    }

    fn kite(&self) -> XY {
        self.tree_view_widget.kite()
    }

    fn get_widget_actions(&self) -> Option<ContextBarItem> {
        Some(ContextBarItem::new_internal_node(
            Cow::Borrowed("file_tree"),
            vec![ContextBarItem::new_leaf_node(
                Cow::Borrowed("toggle hidden files filter"),
                || FileTreeViewMsg::ToggleHiddenFilesFilter.boxed(),
                Some(self.config.keyboard_config.file_tree.toggle_hidden_files),
            )],
        ))
    }
}

impl TreeViewWidget<SPath, FileTreeNode> {
    pub fn expand_path(&mut self, path: &SPath) -> bool {
        debug!("setting path to {}", path);

        let root_node = self.get_root_node();

        if !root_node.spath().is_parent_of(path) {
            error!("attempted to set path {}, but root is {}, ignoring.", path, root_node.spath());
            return false;
        }

        let exp_mut = self.expanded_mut();

        let mut parent_ref_iter = path.ancestors_and_self_ref();
        while let Some(anc) = parent_ref_iter.next() {
            if anc.is_file() {
                continue;
            }

            exp_mut.insert(anc.clone());
        }

        true
    }
}
