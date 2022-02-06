use std::borrow::Borrow;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use log::warn;

use crate::widgets::list_widget::ListWidgetItem;
use crate::widgets::tree_view::tree_view_node::{ChildRc, TreeViewNode};

pub struct FilesystemNode {
    path: PathBuf,
    dir_cache: RefCell<Vec<Rc<FilesystemNode>>>,
}

impl FilesystemNode {
    pub fn new(path: PathBuf) -> FilesystemNode {
        FilesystemNode {
            path,
            dir_cache: RefCell::new(vec![]),
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
                            // if dir_entry.path().is_dir() {
                            let item = Rc::new(FilesystemNode::new(dir_entry.path()));
                            new_cache.push(item.clone());
                            // }
                        }
                    }
                }

                self.dir_cache.replace(new_cache);
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
        self.dir_cache.borrow().len()
    }

    fn get_child(&self, idx: usize) -> ChildRc<PathBuf> {
        // TODO panic
        self.dir_cache.borrow().get(idx).unwrap().clone()
    }

    fn get_child_by_key(&self, key: &PathBuf) -> Option<ChildRc<PathBuf>> {
        for c in self.dir_cache.borrow().iter() {
            if c.id() == key {
                return Some(c.clone());
            }
        }
        None
    }


    fn has_child(&self, key: &PathBuf) -> bool {
        for c in self.dir_cache.borrow().iter() {
            if c.id() == key {
                return true;
            }
        }
        false
    }

    fn todo_update_cache(&self) {
        self.update_cache()
    }
}
