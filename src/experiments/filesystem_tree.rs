use std::borrow::{Borrow, BorrowMut};
use std::fs::ReadDir;
use std::path::{Path, PathBuf};

use log::warn;

use crate::widget::tree_view_node::TreeViewNode;

struct FilesystemNode {
    path: PathBuf,
}

struct FilesystemChildrenIterator<'a> {
    path: &'a Path,
    readdir_op: Option<ReadDir>,
}

impl<'a> Iterator for FilesystemChildrenIterator<'a> {
    type Item = &'a dyn TreeViewNode<PathBuf>;

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
                                    &FilesystemNode {
                                        path: direntry.path()
                                    }
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

    fn children(&self) -> Box<dyn Iterator<Item=&dyn TreeViewNode<PathBuf>> + '_> {
        Box::new(FilesystemChildrenIterator {
            path: self.path.borrow(),
            readdir_op: None,
        })
    }

    fn is_leaf(&self) -> bool {
        false
    }
}