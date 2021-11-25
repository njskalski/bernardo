use std::borrow::{Borrow, BorrowMut};
use std::fs::ReadDir;
use std::path::{Path, PathBuf};

use log::warn;

use crate::widget::tree_view_node::TreeViewNode;

struct FilesystemNode {
    path: PathBuf,
}

struct FilesystemChildrenIterator {
    path: PathBuf,
    readdir_op: Option<ReadDir>,
}

impl Borrow<dyn TreeViewNode<PathBuf>> for FilesystemNode {
    fn borrow(&self) -> &(dyn TreeViewNode<PathBuf> + 'static) {
        self
    }
}

impl Iterator for FilesystemChildrenIterator {
    type Item = Box<dyn Borrow<dyn TreeViewNode<PathBuf>>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.readdir_op {
            None => {
                match self.path.read_dir() {
                    Ok(readdir) => {
                        self.readdir_op = Some(readdir);
                    }
                    Err(err) => {
                        warn!("failed reading directory {:?}, {}", self.path, err);
                    }
                }
            }
            _ => {}
        };

        return match self.readdir_op.borrow_mut() {
            None => None,
            Some(readdir) => {
                match readdir.next() {
                    None => None,
                    Some(res_direntry) => {
                        match res_direntry {
                            Err(err) => {
                                warn!("error reading direntry within {:?}, {}", self.path, err);
                                None
                            }
                            Ok(direntry) => {
                                Some(
                                    Box::new(FilesystemNode {
                                        path: direntry.path()
                                    }) as Box<dyn Borrow<dyn TreeViewNode<PathBuf>>>
                                )
                            }
                        }
                    }
                }
            }
        };
    }
}

impl TreeViewNode<PathBuf> for FilesystemNode {
    fn id(&self) -> &PathBuf {
        self.path.borrow()
    }

    fn label(&self) -> String {
        "whatever".to_string()
    }

    fn children(&self) -> Box<dyn Iterator<Item=Box<dyn Borrow<dyn TreeViewNode<PathBuf>>>>> {
        Box::new(FilesystemChildrenIterator {
            path: self.path.clone(),
            readdir_op: None,
        })
    }

    fn is_leaf(&self) -> bool {
        false
    }
}