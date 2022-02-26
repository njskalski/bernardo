use std::{fmt, iter};
use std::borrow::Borrow;
use std::cell::{BorrowError, Ref, RefCell};
use std::fmt::{Debug, Formatter};
use std::iter::empty;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use log::error;

use crate::io::filesystem_tree::filesystem_front::{FilesystemFront, FsfRef};
use crate::widgets::list_widget::{ListWidgetItem, ListWidgetProvider};
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

#[derive(Clone, Debug)]
pub enum FileType {
    File,
    Directory { cache: Rc<RefCell<FileChildrenCache>> },
}

pub struct FileChildrenCache {
    pub complete: bool,
    pub children: Vec<Rc<FileFront>>,
}

impl Debug for FileChildrenCache {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} cache with {} items",
               if self.complete { "complete" } else { "incomplete" },
               self.children.len(),
        )
    }
}

impl Default for FileChildrenCache {
    fn default() -> Self {
        FileChildrenCache {
            complete: false,
            children: vec![],
        }
    }
}

#[derive(Debug)]
pub struct FileFront {
    path: Rc<PathBuf>,
    file_type: FileType,
}

impl FileFront {
    pub fn new_file(path: Rc<PathBuf>) -> Self {
        Self {
            path,
            file_type: FileType::File,
        }
    }

    pub fn new_directory(path: Rc<PathBuf>, cache: Rc<RefCell<FileChildrenCache>>) -> Self {
        Self {
            path,
            file_type: FileType::Directory {
                cache
            },
        }
    }
}

impl TreeViewNode<PathBuf> for Rc<FileFront> {
    fn id(&self) -> &PathBuf {
        &self.path
    }

    fn label(&self) -> String {
        self.path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or("error".to_string()) //TODO
    }

    fn is_leaf(&self) -> bool {
        match self.file_type {
            FileType::File => true,
            FileType::Directory { .. } => false
        }
    }

    fn num_child(&self) -> (bool, usize) {
        match &self.file_type {
            FileType::File => (true, 0),
            FileType::Directory { cache } => {
                cache.try_borrow().map(
                    |c| (c.complete, c.children.len())
                ).unwrap_or_else(|_| {
                    error!("failed to access cache");
                    (false, 0)
                })
            }
        }
    }

    fn get_child(&self, idx: usize) -> Option<Self> {
        return match &self.file_type {
            FileType::File => None,
            FileType::Directory { cache } => {
                cache.try_borrow().map(
                    |c| c.children.get(idx).map(|f| f.clone())
                ).unwrap_or_else(|_| {
                    error!("failed to access cache");
                    None
                })
            }
        }
    }

    fn is_complete(&self) -> bool {
        match &self.file_type {
            FileType::File => true,
            FileType::Directory { cache } => {
                cache.try_borrow().map(
                    |c| c.complete
                ).unwrap_or_else(|_| {
                    error!("failed to access cache");
                    false
                })
            }
        }
    }
}

impl ListWidgetItem for Rc<FileFront> {
    fn get_column_name(_idx: usize) -> &'static str {
        "name"
    }

    fn get_min_column_width(_idx: usize) -> u16 {
        10
    }

    fn len_columns() -> usize {
        1
    }

    fn get(&self, _idx: usize) -> Option<String> {
        self.path.to_str().map(|f| f.to_string())
    }
}

impl ListWidgetProvider<Rc<FileFront>> for Rc<FileFront> {
    // TODO add loading?

    fn len(&self) -> usize {
        self.num_child().1
    }

    fn get(&self, idx: usize) -> Option<Rc<FileFront>> {
        self.get_child(idx)
    }
}