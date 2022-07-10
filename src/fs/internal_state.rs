use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use filesystem::{FileSystem, OsFileSystem};
use ignore::gitignore::Gitignore;
use log::{error, warn};

use crate::fs::constants::is_sham;
use crate::fs::file_front::{FileChildrenCache, FileChildrenCacheRef};
use crate::io::loading_state::LoadingState;
use crate::widgets::fuzzy_search::helpers::is_subsequence;

// how many file paths should be available for immediate querying "at hand".
// basically a default size of cache for fuzzy file search
const DEFAULT_FILES_PRELOADS: usize = 128 * 1024;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct WrappedArcPath(pub Arc<PathBuf>);

impl Borrow<Path> for WrappedArcPath {
    fn borrow(&self) -> &Path {
        self.0.as_path()
    }
}

impl Borrow<Arc<PathBuf>> for WrappedArcPath {
    fn borrow(&self) -> &Arc<PathBuf> {
        &self.0
    }
}

pub struct InternalState {
    root_path: PathBuf,
    fs: filesystem::OsFileSystem,
    children_cache: HashMap<WrappedArcPath, Arc<RefCell<FileChildrenCache>>>,
    // I need to store some identifier, as search_index.remove requires it. I choose u128 so I can
    // safely not reuse them.
    paths: HashMap<WrappedArcPath, u128>,
    rev_paths: HashMap<u128, WrappedArcPath>,
    pub at_hand_limit: usize, // TODO privatize

    current_idx: u128,

    gitignores: HashMap<WrappedArcPath, ignore::gitignore::Gitignore>,
}

impl InternalState {
    pub fn new(root_path: PathBuf) -> Self {
        InternalState {
            root_path,
            fs: OsFileSystem::new(),
            children_cache: HashMap::default(),
            paths: HashMap::default(),
            rev_paths: HashMap::default(),
            at_hand_limit: DEFAULT_FILES_PRELOADS,
            current_idx: 1,
            gitignores: HashMap::default(),
        }
    }
}

impl Debug for InternalState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "internal_state: {:?} root_path {} paths, {} caches, current_idx = {}", self.root_path, self.paths.len(), self.children_cache.len(), self.current_idx)
    }
}

pub struct FuzzyFileIt<'a> {
    is: &'a InternalState,
    query: String,
    _idx: usize,
}

impl InternalState {
    pub fn get_or_create_cache(&mut self, path: &Arc<PathBuf>) -> FileChildrenCacheRef {
        match self.children_cache.get(path) {
            None => {
                let cache = Arc::new(RefCell::new(FileChildrenCache::default()));
                self.children_cache.insert(WrappedArcPath(path.clone()), cache.clone());
                FileChildrenCacheRef(cache)
            }
            Some(cache) => FileChildrenCacheRef(cache.clone()),
        }
    }

    pub fn get_cache(&self, path: &Path) -> Option<FileChildrenCacheRef> {
        self.children_cache.get(path).map(|f| FileChildrenCacheRef(f.clone()))
    }

    pub fn get_path(&self, path: &Path) -> Option<Arc<PathBuf>> {
        self.paths.get_key_value(path).map(|(p, _)| p.0.clone())
    }

    pub fn remove_path(&mut self, path: &Path) -> bool {
        if !self.paths.contains_key(path) {
            return false;
        }

        let (key, value) = self.paths.get_key_value(path).map(|(a, b)| (a.0.clone(), *b)).unwrap();

        if Arc::strong_count(&key) > 3 {
            warn!("removing path with more than three strong referrers - possible leak?");
        }

        self.paths.remove(path);
        self.rev_paths.remove(&value);
        // self.search_index.delete(&value);

        true
    }

    fn _create_path(&mut self, path: &Path) -> Arc<PathBuf> {
        let rcp = Arc::new(path.to_owned());
        let idx = self.current_idx;
        self.current_idx += 1;

        self.paths.insert(WrappedArcPath(rcp.clone()), idx);
        self.rev_paths.insert(idx, WrappedArcPath(rcp.clone()));

        rcp
    }

    pub fn get_or_create_path(&mut self, path: &Path) -> Arc<PathBuf> {
        if let Some((p, _)) = self.paths.get_key_value(path) {
            p.0.clone()
        } else {
            self._create_path(path)
        }
    }

    pub fn clear_gitignore(&mut self, path: &Path) {
        if !path.starts_with(&self.root_path) {
            error!("clearing gitignore outside the root path: {:?} outside {:?}", path, self.root_path);
            // this is not fatal.
        }

        if self.gitignores.remove(path).is_none() {
            warn!("cleared absent gitignore at {:?}", path);
        }
    }

    pub fn set_gitignore(&mut self, gitignore: Gitignore) {
        if !gitignore.path().starts_with(&self.root_path) {
            error!("attempted to set a gitignore for path outside root path: {:?} outside {:?}", gitignore.path(), self.root_path);
            return;
        }

        let gp = self.get_or_create_path(gitignore.path());
        if self.gitignores.insert(WrappedArcPath(gp.clone()), gitignore).is_some() {
            warn!("replaced gitignore at {:?}", gp);
        }
    }

    pub fn fuzzy_files_it(&self, query: String) -> (LoadingState, Box<dyn Iterator<Item=Arc<PathBuf>> + '_>) {
        // TODO this is dumb as fuck, just to prove rest works
        let iter = self.paths.iter().filter(move |item| {
            item.0.0.file_name().map(|f| f.to_str()).flatten().map(|s| is_subsequence(s, &query)).unwrap_or_else(|| {
                error!("fuzzy_files_it: path is not a utf-8 str");
                false
            })
        }).map(|item| item.0.0.clone());

        //TODO not implemented properly informing on loading state
        (LoadingState::Complete, Box::new(iter))
    }

    /*
    Returns the most significant .gitignore file (one that is deepest on path from path to root).
     */
    pub fn get_gitignore(&self, path: &Path) -> Option<&ignore::gitignore::Gitignore> {
        if !path.starts_with(&self.root_path) {
            warn!("requested gitignore for a path outside root path: {:?}", path);
            return None;
        }
        for p in path.ancestors().skip(1) {
            if !p.starts_with(&self.root_path) {
                break;
            }
            if let Some(x) = self.gitignores.get(p) {
                return Some(x);
            }
        }
        None
    }

    /*
    This should return true for all paths covered by gitignore and analogues of other scms,
    and other sham like ".git", ".idea", ".cache" etc., generally everything until we escalate.
     */
    pub fn is_ignored(&self, path: &Path) -> bool {
        if is_sham(path) {
            return true;
        }

        let is_dir = self.fs.is_dir(path);
        self.get_gitignore(path).map(|gitignore| {
            gitignore.matched_path_or_any_parents(path, is_dir).is_ignore()
        }).unwrap_or(false)
    }
}