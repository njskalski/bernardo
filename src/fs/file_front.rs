use std::{fmt, io};
use std::cell::{BorrowMutError, RefCell};
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use log::{error, warn};
use ron::ser::PrettyConfig;
use ropey::Rope;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::fs::filesystem_front::SomethingToSave;
use crate::fs::FsfRef::FsfRef;
use crate::fs::read_error::ReadError;
use crate::fs::write_error::WriteError;
use crate::io::loading_state::LoadingState;
use crate::widgets::list_widget::{ListWidgetItem, ListWidgetProvider};
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

type FilterType = fn(&SPath) -> bool;

pub struct FileChildrenCache {
    pub loading_state: LoadingState,
    pub children: Vec<Arc<PathBuf>>,
}

/*
Why this exists:
The cache is written on tick (inotify or blocking read) and read by everybody else. This means
it must be Rc<RefCell<>> :(. And I wanted helper methods, which is not possible without wrapping the
non-local types.
 */
pub struct FileChildrenCacheRef(pub Arc<RefCell<FileChildrenCache>>);

impl FileChildrenCacheRef {
    pub fn get_children(&self) -> (LoadingState, Vec<Arc<PathBuf>>) {
        if let Ok(r) = self.0.try_borrow() {
            (r.loading_state, r.children.clone())
        } else {
            warn!("failed to acquire cache ref");
            (LoadingState::Error, vec![])
        }
    }

    pub fn set_loading_state(&self, loading_state: LoadingState) -> Result<(), BorrowMutError> {
        self.0
            .try_borrow_mut()
            .map(|mut c| c.loading_state = loading_state)
    }

    pub fn get_loading_state(&self) -> LoadingState {
        self.0.try_borrow().map(|c| c.loading_state).unwrap_or({
            error!("get_loading_state: failed acquiring lock");
            LoadingState::Error
        })
    }
}

impl Debug for FileChildrenCache {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} cache with {} items",
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

#[derive(Clone, Debug, Hash)]
pub struct SPath {
    // TODO I have not decided or forgot what I decided, whether this path is relative to fsf root or not.
    path: Arc<PathBuf>,
    fsf: FsfRef,
}

impl PartialEq for SPath {
    fn eq(&self, other: &Self) -> bool {
        let same_thing: bool = self.fsf == other.fsf && self.path == other.path;

        if cfg!(debug_assertions) {
            if self.path.as_ref() == other.path.as_ref() && !same_thing {
                error!("found duplicate of PathBuf {:?}, which was supposed to be impossible. We have a leak.", self.path.as_path());
            }
        }

        same_thing
    }
}

impl Eq for SPath {}

impl SPath {
    pub fn new(fsf: FsfRef, path: Arc<PathBuf>) -> SPath {
        Self { path, fsf }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn relative_path(&self) -> &Path {
        // TODO unwrap
        self.path
            .strip_prefix(self.fsf.get_root_path().as_ref())
            .unwrap()
    }

    pub fn path_rc(&self) -> &Arc<PathBuf> {
        &self.path
    }

    pub fn is_dir(&self) -> bool {
        self.fsf.0.is_dir(&self.path)
    }

    pub fn is_file(&self) -> bool {
        self.fsf.0.is_file(&self.path)
    }

    pub fn children(&self) -> Box<dyn Iterator<Item=SPath> + '_> {
        self.fsf.get_children(&self.path).1
    }

    pub fn fsf(&self) -> &FsfRef {
        &self.fsf
    }

    pub fn read_entire_file_to_rope(&self) -> Result<Rope, ReadError> {
        self.fsf.read_entire_file_to_rope(self.path())
    }

    pub fn read_entire_file_to_bytes(&self) -> Result<Vec<u8>, ReadError> {
        self.fsf.read_entire_file_bytes(self.path())
    }

    pub fn read_entire_file_to_item<I: DeserializeOwned>(&self) -> Result<I, ReadError> {
        let bytes = self.read_entire_file_to_bytes()?;
        let item: I = ron::de::from_bytes::<I>(&bytes)?;
        Ok(item)
    }

    pub fn overwrite_item_ron<I: Serialize>(&self, item: &I) -> Result<(), WriteError> {
        let bytes = ron::ser::to_string_pretty(item, PrettyConfig::default())?;
        self.overwrite_with(&bytes.as_str()).map_err(|e| WriteError::IoError(e))
    }

    /*
    Fails only if parent is outside root
     */
    pub fn parent(&self) -> Option<SPath> {
        self.path.parent().map(|f| self.fsf.get_item(f)).flatten()
    }

    pub fn overwrite_with(&self, source: &dyn SomethingToSave) -> Result<(), io::Error> {
        self.fsf.overwrite_file(self.path(), source)
    }

    pub fn display_file_name(&self) -> &str {
        self.path()
            .file_name()
            .map(|oss| {
                oss.to_str().unwrap_or_else(|| {
                    error!("failed to cast path to string: {:?}", self.path());
                    crate::fs::constants::NON_UTF8_ERROR_STR
                })
            })
            .unwrap_or_else(|| {
                error!("failed to extract a filename from: {:?}", self.path());
                crate::fs::constants::NOT_A_FILENAME
            })
    }

    pub fn descendant<T: AsRef<Path> + ?Sized>(&self, suffix: &T) -> Option<SPath> {
        let new_path = self.path().join(suffix);
        if self.fsf.exists(&new_path) {
            self.fsf.get_item(&new_path)
        } else {
            None
        }
    }

    //TODO tests
    pub fn display_last_dir_name(&self, strip_prefix: bool) -> &str {
        let path = if self.is_dir() {
            self.path.as_path()
        } else {
            match self.path.parent() {
                None => self.path.as_path(),
                Some(p) => p,
            }
        };

        let path = if strip_prefix {
            let prefix = self.fsf.get_root_path().as_path();
            match path.strip_prefix(prefix) {
                Ok(p) => p,
                Err(_e) => {
                    warn!("failed stripping prefix {:?} from {:?}", prefix, path);
                    path
                }
            }
        } else {
            path
        };

        path.to_str().unwrap_or_else(|| {
            error!("failed to cast path to string: {:?}", self.path());
            crate::fs::constants::NON_UTF8_ERROR_STR
        })
    }
}

impl TreeViewNode<PathBuf> for SPath {
    fn id(&self) -> &PathBuf {
        &self.path
    }

    fn label(&self) -> String {
        self.path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or("error".to_string()) //TODO
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

impl ListWidgetItem for SPath {
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

        self.path
            .file_name()
            .map(|f| f.to_str().map(|f| f.to_string()))
            .flatten()
            .or(Some("error".to_string()))
    }
}

#[derive(Clone)]
pub struct FilteredSPath {
    ff: SPath,
    filter: FilterType,
}

impl Debug for FilteredSPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[filtered {:?}]", self.ff)
    }
}

impl FilteredSPath {
    pub fn new(ff: SPath, filter: FilterType) -> Self {
        Self { ff, filter }
    }
}

impl ListWidgetProvider<SPath> for FilteredSPath {
    fn len(&self) -> usize {
        self.ff.children().filter(|x| (self.filter)(x)).count()
    }

    fn get(&self, idx: usize) -> Option<SPath> {
        self.ff
            .children()
            .filter(|x| (self.filter)(x))
            .nth(idx)
            .map(|f| f.clone())
    }
}
