use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, Ref, RefCell};
use std::fs::ReadDir;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use log::warn;

use crate::widget::tree_it::TreeIt;
use crate::widget::tree_view_node::TreeViewNode;

pub struct FilesystemNode {
    path: PathBuf,
    cache: Vec<Rc<FilesystemNode>>,
}

impl FilesystemNode {
    pub fn new(path: PathBuf) -> FilesystemNode {
        FilesystemNode {
            path,
            cache: vec![],
        }
    }
}

struct ReadCache {
    read_dir: ReadDir,
    children: Vec<FilesystemNode>,
}


impl FilesystemNode {
    fn update_cache(&mut self) {
        match self.path.read_dir() {
            Err(err) => {
                warn!("failed to read dir {:?}, {}", self.path, err);
            }
            Ok(readdir) => {
                self.cache.clear();

                for dir_entry_op in readdir {
                    match dir_entry_op {
                        Err(err) => {
                            warn!("failed to get dir_entry in {:?}, {}", self.path, err);
                            // TODO add error item?
                        }
                        Ok(dir_entry) => {
                            let item = Rc::new(FilesystemNode::new(dir_entry.path()));
                            self.cache.push(item.clone());
                        }
                    }
                }
            }
        }
    }
}

impl TreeViewNode<PathBuf> for FilesystemNode {
    fn id(&self) -> &PathBuf {
        self.path.borrow()
    }

    fn label(&self) -> String {
        "whatever".to_string()
    }
    

    fn is_leaf(&self) -> bool {
        false
    }

    fn num_child(&self) -> usize {
        self.cache.len()
    }

    fn get_child(&self, idx: usize) -> &dyn TreeViewNode<PathBuf> {
        todo!()
    }
}
