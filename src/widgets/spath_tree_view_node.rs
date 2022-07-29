use std::fmt::{Debug, Formatter};
use log::error;
use crate::new_fs::dir_entry::DirEntry;
use crate::new_fs::path::SPath;
use crate::new_fs::read_error::ListError;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

#[derive(Debug, Clone)]
pub struct DirTreeNode {sp : SPath}

impl DirTreeNode {
    pub fn new(sp : SPath) -> Self {
        DirTreeNode { sp }
    }

    pub fn spath(&self) -> &SPath {
        &self.sp
    }
}

#[derive(Debug, Clone)]
pub struct FileTreeNode{ sp : SPath }

impl FileTreeNode {
    pub fn new(sp : SPath) -> Self {
        FileTreeNode { sp }
    }

    pub fn spath(&self) -> &SPath {
        &self.sp
    }
}

impl TreeViewNode<SPath> for FileTreeNode {
    fn id(&self) -> &SPath {
        &self.sp
    }

    fn label(&self) -> String {
        self.sp.label()
    }

    fn is_leaf(&self) -> bool {
        self.sp.is_file()
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item=Self>> {
        match self.sp.blocking_list() {
            Ok(items) => Box::new(
                items.into_iter().map(|item| FileTreeNode::new(item))
            ) as Box<dyn Iterator<Item=Self>>,
            Err(e) => {
                error!("fail to call blocking_list {:?}", e);
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item=Self>>
            }
        }
    }

    fn is_complete(&self) -> bool {
        true //TODO
    }
}

impl TreeViewNode<SPath> for DirTreeNode {
    fn id(&self) -> &SPath {
        &self.sp
    }

    fn label(&self) -> String { self.sp.label() }

    fn is_leaf(&self) -> bool {
        self.sp.is_file()
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item=Self>> {
        match self.sp.blocking_list() {
            Ok(items) => Box::new(
                items.into_iter().filter(|c| c.is_dir()).map(|item| DirTreeNode::new(item))
            ) as Box<dyn Iterator<Item=Self>>,
            Err(e) => {
                error!("fail to call blocking_list {:?}", e);
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item=Self>>
            }
        }
    }

    fn is_complete(&self) -> bool {
        true //TODO
    }
}