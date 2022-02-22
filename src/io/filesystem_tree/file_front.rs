use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use log::error;

use crate::io::filesystem_tree::filesystem_front::{FilesystemFront, FsfRef};
use crate::widgets::list_widget::ListWidgetItem;
use crate::widgets::tree_view::tree_view_node::{ChildRc, TreeViewNode};

struct Cache {
    complete: bool,
    children: Vec<Rc<FileFront>>,
}

#[derive(Debug)]
pub struct FileFront {
    path: PathBuf,
    fsf: FsfRef,

    dir_cache: RefCell<Cache>,
}

impl FileFront {
    fn update_cache(&self) {
        let (complete, children) = self.fsf.get_children(&self.path);

        let mut child_vec: Vec<Rc<FileFront>> = vec![];
        for c in children {
            child_vec.push(c);
        }

        self.dir_cache.replace(
            Cache {
                complete,
                children: child_vec,
            }
        );
    }
}

impl TreeViewNode<PathBuf> for FileFront {
    fn id(&self) -> &PathBuf {
        &self.path
    }

    fn label(&self) -> String {
        self.path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or("error".to_string()) //TODO
    }

    fn is_leaf(&self) -> bool {
        !self.fsf.is_dir(&self.path) //TODO
    }

    fn num_child(&self) -> (bool, usize) {
        self.update_cache();
        self.dir_cache.try_borrow().map(
            |c| (c.complete, c.children.len())
        ).unwrap_or_else(|_| {
            error!("failed to access cache");
            (false, 0)
        })
    }

    fn get_child(&self, idx: usize) -> Option<ChildRc<PathBuf>> {
        self.dir_cache.try_borrow().map(
            |c| c.children.get(idx).map(|f| f.clone() as Rc<dyn TreeViewNode<PathBuf>>)
        ).unwrap_or_else(|_| {
            error!("failed to access cache");
            None
        })
    }

    fn get_child_by_key(&self, key: &PathBuf) -> Option<ChildRc<PathBuf>> {
        self.dir_cache.try_borrow().map(
            |c| {
                for child in c.children.iter() {
                    if child.path == *key {
                        return Some(child.clone() as Rc<dyn TreeViewNode<PathBuf>>);
                    }
                }
                None
            })
            .unwrap_or_else(|_| {
                error!("failed to access cache");
                None
            })
    }

    fn is_complete(&self) -> bool {
        self.dir_cache.try_borrow().map(
            |c| c.complete
        ).unwrap_or_else(|_| {
            error!("failed to access cache");
            false
        })
    }

    fn children(&self) -> (bool, Box<dyn Iterator<Item=ChildRc<PathBuf>>>) {
        self.dir_cache.try_borrow().map(
            |c| (c.complete, Box::new(c.children.iter()))
        ).unwrap_or_else(|_| {
            error!("failed to access cache");
            (false, Box::new(vec![].iter()))
        })
    }
}

impl ListWidgetItem for FileFront {
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
