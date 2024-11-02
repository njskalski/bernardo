use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::Arc;

use crate::primitives::tree::tree_node::TreeNode;
use crate::widgets::context_menu::widget::ContextMenuWidget;
use crate::widgets::main_view::display::MainViewDisplay;

// TODO this class is "a quick fix" to a problem I don't want to think about now.

#[derive(Debug, Clone)]
pub enum Type {
    Root(Vec<Arc<DisplayRegistryItem>>),
    Buffer { description: String },
    BufferList(Vec<Arc<DisplayRegistryItem>>),
    CodeResults { description: String },
    CodeResultsList(Vec<Arc<DisplayRegistryItem>>),
}
#[derive(Debug, Clone)]
pub struct DisplayRegistryItem {
    id: usize,
    t: Type,
}

pub type DisplayRegistryTreeNode = Arc<DisplayRegistryItem>;

impl TreeNode<usize> for DisplayRegistryTreeNode {
    fn id(&self) -> &usize {
        &self.id
    }

    fn label(&self) -> Cow<str> {
        match &self.t {
            Type::Root(_) => Cow::Borrowed("root"),
            Type::Buffer { description } => Cow::Borrowed(description.as_ref()),
            Type::CodeResults { description } => Cow::Borrowed(description.as_ref()),
            Type::BufferList(_) => Cow::Borrowed("buffers:"),
            Type::CodeResultsList(_) => Cow::Borrowed("search/code results views:"),
        }
    }

    fn is_leaf(&self) -> bool {
        match &self.t {
            Type::Root(_) => false,
            Type::BufferList(_) => false,
            Type::CodeResultsList(_) => false,
            _ => true,
        }
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item=Self> + '_> {
        match &self.t {
            Type::Root(items) => Box::new(items.clone().into_iter()) as Box<dyn Iterator<Item=Self> + '_>,
            Type::Buffer { .. } => Box::new(std::iter::empty()),
            Type::BufferList(items) => Box::new(items.clone().into_iter()),
            Type::CodeResults { .. } => Box::new(std::iter::empty()),
            Type::CodeResultsList(items) => Box::new(items.clone().into_iter()),
        }
    }

    fn is_complete(&self) -> bool {
        true
    }
}

pub type FuzzyScreensList = ContextMenuWidget<usize, DisplayRegistryTreeNode>;

pub fn get_fuzzy_screen_list(displays: &Vec<MainViewDisplay>, display_idx: usize) -> DisplayRegistryTreeNode {
    let mut buffer_list: Vec<Arc<DisplayRegistryItem>> = Vec::new();
    let mut results_view_list: Vec<Arc<DisplayRegistryItem>> = Vec::new();

    for (idx, display) in displays.iter().enumerate() {
        match display {
            MainViewDisplay::Editor(e) => {
                let buf = DisplayRegistryItem {
                    id: idx,
                    t: Type::Buffer {
                        description: "x".to_string(),
                    },
                };

                buffer_list.push(Arc::new(buf));
            }
            MainViewDisplay::ResultsView(_) => {
                let code_view = DisplayRegistryItem {
                    id: idx,
                    t: Type::CodeResults {
                        description: "y".to_string(),
                    },
                };

                results_view_list.push(Arc::new(code_view));
            }
        }
    }

    let mut items: Vec<Arc<DisplayRegistryItem>> = Vec::new();
    if buffer_list.is_empty() == false {
        items.push(Arc::new(DisplayRegistryItem {
            id: display_idx + 1,
            t: Type::BufferList(buffer_list),
        }));
    }
    if results_view_list.is_empty() == false {
        items.push(Arc::new(DisplayRegistryItem {
            id: display_idx + 2,
            t: Type::CodeResultsList(results_view_list),
        }));
    }

    Arc::new(DisplayRegistryItem {
        id: display_idx,
        t: Type::Root(items),
    })
}
