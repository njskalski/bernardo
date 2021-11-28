use std::borrow::Borrow;
use std::cell::RefCell;
use std::fs::ReadDir;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;

use log::warn;

use crate::widgets::tree_view::tree_view_node::{ChildRc, TreeViewNode};

pub struct FilesystemNode {
    path: PathBuf,
    cache: RefCell<Vec<Rc<FilesystemNode>>>,
}

impl FilesystemNode {
    pub fn new(path: PathBuf) -> FilesystemNode {
        FilesystemNode {
            path,
            cache: RefCell::new(vec![]),
        }
    }
}


impl FilesystemNode {
    pub fn update_cache(&self) {
        match self.path.read_dir() {
            Err(err) => {
                warn!("failed to read dir {:?}, {}", self.path, err);
            }
            Ok(readdir) => {
                let mut new_cache: Vec<Rc<FilesystemNode>> = vec![];

                for dir_entry_op in readdir {
                    match dir_entry_op {
                        Err(err) => {
                            warn!("failed to get dir_entry in {:?}, {}", self.path, err);
                            // TODO add error item?
                        }
                        Ok(dir_entry) => {
                            let item = Rc::new(FilesystemNode::new(dir_entry.path()));
                            new_cache.push(item.clone());
                        }
                    }
                }

                self.cache.replace(new_cache);
            }
        }
    }
}

impl TreeViewNode<PathBuf> for FilesystemNode {
    fn id(&self) -> &PathBuf {
        self.path.borrow()
    }

    fn label(&self) -> String {
        self.path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or("error".to_string())
    }

    fn is_leaf(&self) -> bool {
        !self.path.is_dir()
    }

    fn num_child(&self) -> usize {
        self.cache.borrow().len()
    }

    fn get_child(&self, idx: usize) -> ChildRc<PathBuf> {
        // TODO panic
        self.cache.borrow().get(idx).unwrap().clone()
    }


    fn has_child(&self, key: &PathBuf) -> bool {
        for c in self.cache.borrow().iter() {
            if c.id() == key {
                return true;
            }
        }
        false
    }
}
