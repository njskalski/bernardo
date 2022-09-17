use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::Arc;

use log::error;
use streaming_iterator::StreamingIterator;

use crate::fs::path::SPath;
use crate::widgets::list_widget::list_widget_provider::ListWidgetProvider;
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
    arc_vec: Arc<Vec<SPath>>,
    current_idx: Option<usize>,
    current_item: Option<FileTreeNode>,
}

impl ArcVecWrapperFile {
    pub fn new(arc_vec: Arc<Vec<SPath>>) -> Self {
        Self {
            arc_vec,
            current_idx: None,
            current_item: None,
        }
    }
}

impl StreamingIterator for ArcVecWrapperFile {
    type Item = FileTreeNode;

    fn advance(&mut self) {
        match self.current_idx {
            None => {
                self.current_idx = Some(0)
            }
            Some(i) => {
                self.current_idx = Some(i + 1)
            }
        }
        let idx = self.current_idx.unwrap();

        self.current_item = self.arc_vec.get(idx).map(|sp| FileTreeNode::new(sp))
    }

    fn get(&self) -> Option<&Self::Item> {
        self.current_item.as_ref()
    }
}

struct ArcVecWrapperDir {
    arc_vec: Arc<Vec<SPath>>,
    current_idx: Option<usize>,
    current_item: Option<DirTreeNode>,
}

impl ArcVecWrapperDir {
    pub fn new(arc_vec: Arc<Vec<SPath>>) -> Self {
        Self {
            arc_vec,
            current_idx: None,
            current_item: None,
        }
    }
}

impl StreamingIterator for ArcVecWrapperDir {
    type Item = DirTreeNode;

    fn advance(&mut self) {
        match self.current_idx {
            None => {
                self.current_idx = Some(0)
            }
            Some(i) => {
                self.current_idx = Some(i + 1)
            }
        }

        let mut idx = self.current_idx.unwrap();
        loop {
            if self.arc_vec.get(idx).map(|item| item.is_dir()).unwrap_or(true) {
                break;
            }
            idx += 1;
        }
        self.current_idx = Some(idx);
        self.current_item = self.arc_vec.get(idx).map(|sp| DirTreeNode::new(sp))
    }

    fn get(&self) -> Option<&Self::Item> {
        self.current_item.as_ref()
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

    fn child_iter(&self) -> Box<dyn StreamingIterator<Item=Self>> {
        match self.sp.blocking_list() {
            // TODO remove that clone
            Ok(items) => Box::new(ArcVecWrapperFile::new(items)),
            Err(e) => {
                error!("fail to call blocking_list {:?}", e);
                Box::new(streaming_iterator::empty()) as Box<dyn StreamingIterator<Item=Self>>
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

    fn child_iter(&self) -> Box<dyn StreamingIterator<Item=Self>> {
        match self.sp.blocking_list() {
            Ok(items) => Box::new(
                ArcVecWrapperDir::new(items)
            ) as Box<dyn StreamingIterator<Item=Self>>,
            Err(e) => {
                error!("fail to call blocking_list {:?}", e);
                Box::new(streaming_iterator::empty()) as Box<dyn StreamingIterator<Item=Self>>
            }
        }
    }

    fn is_complete(&self) -> bool {
        true //TODO
    }
}