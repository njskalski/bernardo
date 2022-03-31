use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use log::error;
use crate::io::filesystem_tree::fsfref::FsfRef;

use crate::widgets::list_widget::{ListWidgetItem, ListWidgetProvider};
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

type FilterType = fn(&FileFront) -> bool;

pub struct FileChildrenCache {
    pub complete: bool,
    pub children: Vec<FileFront>,
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FileFront {
    path: Rc<PathBuf>,
    fsf: FsfRef,
}

impl FileFront {
    pub fn new(fsf: FsfRef, path: Rc<PathBuf>) -> FileFront {
        Self {
            path,
            fsf,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_dir(&self) -> bool {
        self.fsf.0.is_dir(&self.path)
    }

    pub fn is_file(&self) -> bool {
        self.fsf.0.is_file(&self.path)
    }

    pub fn children(&self) -> Box<dyn Iterator<Item=FileFront>> {
        self.fsf.0.get_children(&self.path).1.into_iter()
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
        self.is_file()
    }

    fn num_child(&self) -> (bool, usize) {
        if self.is_file() {
            (true, 0)
        } else {
            let (done, items) = self.fsf.0.get_children(&self.path);
            (done, items.count())
        }
    }

    fn get_child(&self, idx: usize) -> Option<Self> {
        self.fsf.0.get_children(&self.path).1.nth(idx)
    }

    fn is_complete(&self) -> bool {
        self.fsf.0.get_children(&self.path).0
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
        if _idx > 0 {
            return None;
        }

        self.path.file_name().map(|f| f.to_str().map(|f| f.to_string())).flatten().or(Some("error".to_string()))
    }
}

#[derive(Clone)]
pub struct FilteredFileFront {
    ff: FileFront,
    filter: FilterType,
}

impl Debug for FilteredFileFront {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[filtered {:?}]", self.ff)
    }
}

impl FilteredFileFront {
    pub fn new(ff: FileFront, filter: FilterType) -> Self {
        Self {
            ff,
            filter,
        }
    }
}

impl ListWidgetProvider<FileFront> for FilteredFileFront {
    fn len(&self) -> usize {
        self.ff.children().filter(|x| (self.filter)(x)).count()
    }

    fn get(&self, idx: usize) -> Option<FileFront> {
        self.ff.children().filter(|x| (self.filter)(x)).nth(idx).map(|f| f.clone())
    }
}
