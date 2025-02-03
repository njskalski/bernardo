use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::Debug;

use crate::fs::fsf_async_tree_iter::FsAsyncTreeIt;
use crate::fs::path::SPath;
use crate::primitives::common_query::CommonQuery;
use crate::primitives::tree::filter_policy::FilterPolicy;
use crate::primitives::tree::tree_node::{FilterRef, TreeItFilter, TreeNode};
use crate::promise::streaming_promise::StreamingPromise;
use log::error;

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

    fn child_iter(&self) -> Box<dyn Iterator<Item = Self> + '_> {
        match self.sp.blocking_list() {
            Ok(items) => Box::new(items.into_iter().map(FileTreeNode::new)) as Box<dyn Iterator<Item = Self>>,
            Err(e) => {
                error!("fail to call blocking_list for {:?} because {:?}", self, e);
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = Self>>
            }
        }
    }

    fn is_complete(&self) -> bool {
        true //TODO
    }

    fn get_streaming_promise_instead_of_iterator(
        &self,
        filter_op: Option<(FilterRef<FileTreeNode>, FilterPolicy)>,
        expanded_op: Option<HashSet<SPath>>,
    ) -> Option<Box<dyn StreamingPromise<(u16, Self)>>> {
        Some(Box::new(FsAsyncTreeIt::new(self.clone(), filter_op, expanded_op)))
        // TODO add expanded
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

    fn child_iter(&self) -> Box<dyn Iterator<Item = Self> + '_> {
        match self.sp.blocking_list() {
            Ok(items) => Box::new(items.into_iter().filter(|c| c.is_dir()).map(DirTreeNode::new)) as Box<dyn Iterator<Item = Self>>,
            Err(e) => {
                error!("fail to call blocking_list for {:?} because {:?}", self, e);
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = Self>>
            }
        }
    }

    fn is_complete(&self) -> bool {
        true //TODO
    }
}

impl TreeItFilter<FileTreeNode> for CommonQuery {
    fn call(&self, node: &FileTreeNode) -> bool {
        let label = node.sp.label();
        self.matches(label.as_ref())
    }
}

impl TreeItFilter<DirTreeNode> for CommonQuery {
    fn call(&self, node: &DirTreeNode) -> bool {
        let label = node.sp.label();
        self.matches(label.as_ref())
    }
}
