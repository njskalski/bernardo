use std::borrow::{Borrow, Cow};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use jsonrpc_core::futures::stream::iter;
use log::error;
use owning_ref::OwningRef;

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

// #[derive(Clone, Debug)]
pub type NewFileTreeNode<const just_dirs: bool> = SPath;


impl TreeViewNode<SPath> for NewFileTreeNode<true> {
    fn id(&self) -> &SPath {
        todo!()
    }

    fn label(&self) -> Cow<str> {
        todo!()
    }

    fn is_leaf(&self) -> bool {
        todo!()
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item=&Self> + '_> {
        match self.blocking_list().map(Deref::deref) {
            Ok(items) => {
                Box::new(items.iter())
            }
            Err(e) => {
                error!("fail to call blocking_list {:?}", e);
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item=&Self>>
            }
        }
    }


    fn is_complete(&self) -> bool {
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

    fn child_iter(&self) -> Box<dyn Iterator<Item=&Self> + '_> {
        todo!()
        // match self.sp.blocking_list_2() {
        //     Ok(items) => items,
        //     Err(e) => {
        //         error!("fail to call blocking_list {:?}", e);
        //         Box::new(std::iter::empty()) as Box<dyn Iterator<Item=&Self>>
        //     }
        // }
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

    fn is_leaf(&self) -> bool {
        self.sp.is_file()
    }

    fn child_iter(&self) -> Box<dyn Iterator<Item=&Self> + '_> {
        todo!()
        // match self.sp.blocking_list() {
        //     Ok(items) => Box::new(
        //         items.into_iter().filter(|c| c.is_dir()).map(|item| &DirTreeNode::new(item))
        //     ) as Box<dyn Iterator<Item=&Self>>,
        //     Err(e) => {
        //         error!("fail to call blocking_list {:?}", e);
        //         Box::new(std::iter::empty()) as Box<dyn Iterator<Item=&Self>>
        //     }
        // }
    }

    fn is_complete(&self) -> bool {
        true //TODO
    }
}