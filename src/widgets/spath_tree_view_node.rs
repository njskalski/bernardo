use std::borrow::Cow;
use std::fmt::Debug;

use log::error;

use crate::fs::path::SPath;
use crate::primitives::tree::tree_node::TreeNode;


#[derive(Debug, Clone)]
pub struct DirTreeNode {
    sp: SPath,
}

impl DirTreeNode {
    pub fn new(sp: SPath) -> Self {
        DirTreeNode { sp }
    }

    pub fn spath(&self) -> &SPath {
        &self.sp
    }
}

#[derive(Debug, Clone)]
pub struct FileTreeNode {
    sp: SPath,
}

impl FileTreeNode {
    pub fn new(sp: SPath) -> Self {
        FileTreeNode { sp }
    }

    pub fn spath(&self) -> &SPath {
        &self.sp
    }
}

impl TreeNode<SPath> for FileTreeNode {
    fn id(&self) -> &SPath {
        &self.sp
    }

    fn label(&self) -> Cow<str> {
        self.sp.label()
    }

    fn is_leaf(&self) -> bool {
        self.sp.is_file()
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item = Self>> {
        match self.sp.blocking_list() {
            Ok(items) => Box::new(items.into_iter().map(FileTreeNode::new)) as Box<dyn Iterator<Item = Self>>,
            Err(e) => {
                error!("fail to call blocking_list {:?}", e);
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = Self>>
            }
        }
    }

    fn is_complete(&self) -> bool {
        true //TODO
    }
}

impl TreeNode<SPath> for DirTreeNode {
    fn id(&self) -> &SPath {
        &self.sp
    }

    fn label(&self) -> Cow<str> {
        self.sp.label()
    }

    fn is_leaf(&self) -> bool {
        self.child_iter().next().is_none()
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item = Self>> {
        match self.sp.blocking_list() {
            Ok(items) => Box::new(items.into_iter().filter(|c| c.is_dir()).map(DirTreeNode::new)) as Box<dyn Iterator<Item = Self>>,
            Err(e) => {
                error!("fail to call blocking_list {:?}", e);
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = Self>>
            }
        }
    }

    fn is_complete(&self) -> bool {
        true //TODO
    }
}
