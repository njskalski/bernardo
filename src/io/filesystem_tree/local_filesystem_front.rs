use std::borrow::Borrow;
use std::fs::{DirEntry, ReadDir};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::Utf8Error;

use filesystem::{FileSystem, OsFileSystem};
use log::{debug, error, warn};
use ropey::Rope;

use crate::io::filesystem_tree::file_front::FileFront;
use crate::io::filesystem_tree::filesystem_front::FilesystemFront;
use crate::text::buffer_state::BufferState;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

#[derive(Debug, Clone)]
pub struct LocalFilesystemFront {
    root: PathBuf,
    fs: OsFileSystem,
}

impl LocalFilesystemFront {
    pub fn new(root: PathBuf) -> Self {
        LocalFilesystemFront {
            root,
            fs: OsFileSystem::new(),
        }
    }

    // substitutes current node corresponding to path with one where children cache is filled.
    // if path is invalid in current tree, it fails.
    // pub fn expand_last(&mut self, path: &Path) -> bool {
    //     // TODO here I am assuming that self.root is prefix to path. This should be checked.
    //
    //     if !path.starts_with(self.root.as_path()) {
    //         warn!("path {:?} is not prefixed by root {:?}", path, self.root)
    //     }
    //
    //     let skip = self.root.components().count();
    //
    //     let mut curr_node = self.root_node.clone() as Rc<dyn TreeViewNode<PathBuf>>;
    //     let mut curr_prefix = self.root.clone();
    //
    //     let num_components = path.components().count();
    //
    //     debug!("comp : {:?}", path.components());
    //
    //     for (idx, c) in path.components().enumerate().skip(skip) {
    //         let _last = idx == num_components - 1;
    //         curr_prefix.push(c);
    //
    //         match curr_node.get_child_by_key(curr_prefix.borrow()) {
    //             None => {
    //                 warn!("{:?} has no child {:?}!", curr_node.id(), curr_prefix);
    //                 return false;
    //             }
    //             Some(new_node) => {
    //                 curr_node = new_node;
    //             }
    //         }
    //     }
    //
    //     // if we got here, curr_node points to node corresponding to path.
    //     debug_assert!(curr_node.id() == path);
    //     curr_node.todo_update_cache();
    //
    //     true
    // }
}

impl FilesystemFront for LocalFilesystemFront {
    fn get_root(&self) -> Rc<dyn TreeViewNode<PathBuf>> {
        todo!()
    }

    fn get_file(&self, path: &Path) -> Option<FileFront> {
        todo!()
    }

    fn todo_read_file(&self, path: &Path) -> Result<Rope, ()> {
        todo!()
    }

    fn is_dir(&self, path: &Path) -> bool {
        todo!()
    }

    fn get_children(&self, path: &Path) -> (bool, Box<dyn Iterator<Item=Rc<FileFront>>>) {
        todo!()
    }
    // fn get_root(&self) -> Rc<dyn TreeViewNode<PathBuf>> {
    //     self.root_node.clone()
    // }
    //
    // // fn expand(&mut self, path: &Path) -> bool {
    // //     self.expand_last(path)
    // // }
    //
    // fn get_files(&self, path: &Path) -> Box<dyn Iterator<Item=FilesystemListItem>> {
    //     if !path.is_dir() {
    //         warn!("path {:?} is not directory.", path);
    //     }
    //
    //     let mut res = vec![];
    //
    //     return match self.fs.read_dir(path) {
    //         Err(err) => {
    //             warn!("failed to read {:?} because {}", path, err);
    //             Box::new(std::iter::empty())
    //         },
    //         Ok(readdir) => {
    //             for c in readdir {
    //                 match c {
    //                     Err(err) => {
    //                         warn!("failed to read {}", err);
    //                     }
    //                     Ok(dir_entry) => {
    //                         if dir_entry.path().is_file() {
    //                             res.push(FilesystemListItem::new(dir_entry.path()))
    //                         }
    //                     }
    //                 }
    //             }
    //             Box::new(res.into_iter())
    //         }
    //     }
    // }
    //
    // fn todo_read_file(&self, path: &Path) -> Result<Rope, ()> {
    //     match self.fs.read_file(path) {
    //         Ok(v8) => {
    //             match std::str::from_utf8(v8.borrow()) {
    //                 Ok(s) => {
    //                     Ok(Rope::from(s))
    //                 }
    //                 Err(e) => {
    //                     error!("file read error {:?} : {}", path, e);
    //                     Err(())
    //                 }
    //             }
    //         }
    //         Err(e) => {
    //             error!("file read error {:?} : {}", path, e);
    //             Err(())
    //         }
    //     }
    // }
}
