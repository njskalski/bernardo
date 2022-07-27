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
        match self.sp.last_name() {
            None => {
                error!("no last_name in SPath used in TreeViewNode");
                "<error>".to_string()
            }
            Some(item) => item.to_string(),
        }
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

    fn num_child(&self) -> (bool, usize) {
        match self.sp.blocking_list() {
            Ok(list) => (true, list.len()),
            Err(e) => {
                error!("failed to list: {:?}", e);
                (false, 0)
            }
        }
    }

    fn get_child(&self, idx: usize) -> Option<Self> {
        todo!()
    }

    fn is_complete(&self) -> bool {
        true //TODO
    }
}

impl TreeViewNode<SPath> for DirTreeNode {
    fn id(&self) -> &SPath {
        &self.sp
    }

    fn label(&self) -> String {
        match self.sp.last_name() {
            None => {
                error!("no last_name in SPath used in TreeViewNode");
                "<error>".to_string()
            }
            Some(item) => item.to_string(),
        }
    }

    fn is_leaf(&self) -> bool {
        self.sp.is_file()
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item=Self>> {
        todo!()
    }

    fn num_child(&self) -> (bool, usize) {
        match self.sp.blocking_list() {
            Ok(list) => (true, list.len()),
            Err(e) => {
                error!("failed to list: {:?}", e);
                (false, 0)
            }
        }
    }

    fn get_child(&self, idx: usize) -> Option<Self> {
        todo!()
    }

    fn is_complete(&self) -> bool {
        true //TODO
    }
}