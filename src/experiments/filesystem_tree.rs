use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fs::ReadDir;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use log::warn;

use crate::widget::tree_it::TreeIt;
use crate::widget::tree_view_node::TreeViewNode;

struct FilesystemNode {
    path: PathBuf,
    cache: RefCell<Option<ReadCache>>,
}

impl FilesystemNode {
    pub fn new(path: PathBuf) -> FilesystemNode {
        FilesystemNode {
            path,
            cache: RefCell::new(None),
        }
    }
}

impl AsRef<dyn TreeViewNode<PathBuf>> for FilesystemNode {
    fn as_ref(&self) -> &(dyn TreeViewNode<PathBuf> + 'static) {
        self
    }
}

struct ReadCache {
    read_dir: ReadDir,
    children: Vec<FilesystemNode>,
}

impl ReadCache {
    fn items_as_ref(&self) -> impl Iterator<Item=&dyn TreeViewNode<PathBuf>> {
        self.children.iter().map(|c| c.as_generic())
    }
}

struct FilesystemChildrenIterator<'a> {
    path: PathBuf,
    _marker: PhantomData<&'a ()>,
    cache: RefCell<Option<ReadCache>>,
}

impl FilesystemNode {
    fn update_cache(&self) {
        match self.path.read_dir() {
            Err(err) => {
                warn!("failed to read dir {:?}, {}", self.path, err);
            }
            Ok(readdir) => {
                let mut items = vec![];
                for dir_entry_op in readdir {
                    match dir_entry_op {
                        Err(err) => {
                            warn!("failed to get dir_entry in {:?}, {}", self.path, err);
                            // TODO add error item?
                        }
                        Ok(dir_entry) => {
                            items.push(FilesystemNode::new(dir_entry.path()));
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

    fn children<'a>(&'a self) -> Box<(dyn Iterator<Item=&'a (dyn TreeViewNode<PathBuf> + 'a)> + 'a)> {
        self.update_cache(); // TODO this should be lazy
        Box::new(self.cache.borrow().map(|x| x.items_as_ref()).unwrap()
        ) // TODO this can fail
    }

    fn is_leaf(&self) -> bool {
        false
    }
}
