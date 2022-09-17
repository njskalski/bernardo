use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::Arc;

use log::error;

use crate::fs::path::SPath;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

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

struct ArcVecWrapperFile {
    pos: usize,
    arc_vec: Arc<Vec<SPath>>,
}

impl ArcVecWrapperFile {
    pub fn new(arc_vec: Arc<Vec<SPath>>) -> Self {
        Self {
            pos: 0,
            arc_vec,
        }
    }
}

impl Iterator for ArcVecWrapperFile {
    type Item = FileTreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

struct ArcVecWrapperDir {
    pos: usize,
    arc_vec: Arc<Vec<SPath>>,
}

impl ArcVecWrapperDir {
    pub fn new(arc_vec: Arc<Vec<SPath>>) -> Self {
        Self {
            pos: 0,
            arc_vec,
        }
    }
}

impl Iterator for ArcVecWrapperDir {
    type Item = DirTreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl TreeViewNode<SPath> for FileTreeNode {
    fn id(&self) -> &SPath {
        &self.sp
    }

    fn label(&self) -> Cow<str> {
        self.sp.label()
    }

    fn is_leaf(&self) -> bool {
        self.sp.is_file()
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item=Self>> {
        match self.sp.blocking_list() {
            // TODO remove that clone
            Ok(items) => Box::new(ArcVecWrapperFile::new(items)),
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

    fn label(&self) -> Cow<str> { self.sp.label() }

    // TODO this is not right
    fn is_leaf(&self) -> bool {
        self.sp.is_file()
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item=Self>> {
        match self.sp.blocking_list() {
            Ok(items) => Box::new(
                ArcVecWrapperDir::new(items)
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