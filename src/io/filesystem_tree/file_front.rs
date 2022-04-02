use std::cell::{BorrowMutError, Ref, RefCell};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use log::{error, warn};
use crate::io::filesystem_tree::fsfref::FsfRef;
use crate::io::filesystem_tree::LoadingState;

use crate::widgets::list_widget::{ListWidgetItem, ListWidgetProvider};
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

type FilterType = fn(&FileFront) -> bool;

pub struct FileChildrenCache {
    pub loading_state: LoadingState,
    pub children: Vec<Rc<PathBuf>>,
}

/*
Why this exists:
The cache is written on tick (inotify or blocking read) and read by everybody else. This means
it must be Rc<RefCell<>> :(. And I wanted helper methods, which is not possible without wrapping the
non-local types.
 */
pub struct FileChildrenCacheRef(pub Rc<RefCell<FileChildrenCache>>);

impl FileChildrenCacheRef {
    pub fn get_children(&self) -> (LoadingState, Vec<Rc<PathBuf>>) {
        if let Ok(r) = self.0.try_borrow() {
            (r.loading_state, r.children.clone())
        } else {
            warn!("failed to acquire cache ref");
            (LoadingState::Error, vec![])
        }
    }

    pub fn set_loading_state(&self, loading_state: LoadingState) -> Result<(), BorrowMutError> {
        self.0.try_borrow_mut().map(|mut c| c.loading_state = loading_state)
    }

    pub fn get_loading_state(&self) -> LoadingState {
        self.0.try_borrow().map(|c| c.loading_state).unwrap_or({
            error!("failed acquiring lock");
            LoadingState::Error
        })
    }
}

impl Debug for FileChildrenCache {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} cache with {} items",
               self.loading_state,
               self.children.len(),
        )
    }
}

impl Default for FileChildrenCache {
    fn default() -> Self {
        FileChildrenCache {
            loading_state: LoadingState::NotStarted,
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

    pub fn path_rc(&self) -> &Rc<PathBuf> { &self.path }

    pub fn is_dir(&self) -> bool {
        self.fsf.0.is_dir(&self.path)
    }

    pub fn is_file(&self) -> bool {
        self.fsf.0.is_file(&self.path)
    }

    pub fn children(&self) -> Box<dyn Iterator<Item=FileFront> + '_> {
        self.fsf.get_children(&self.path).1
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
            let (loading_state, items) = self.fsf.get_children(&self.path);
            //TODO escalate the LoadingState

            let done = match loading_state {
                LoadingState::Complete => true,
                _ => false,
            };

            (done, items.count())
        }
    }

    fn get_child(&self, idx: usize) -> Option<Self> {
        self.fsf.get_children(&self.path).1.nth(idx)
    }

    fn is_complete(&self) -> bool {
        self.fsf.get_children(&self.path).0 == LoadingState::Complete
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
