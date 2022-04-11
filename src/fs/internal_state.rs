use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use log::{error, warn};
use simsearch::{SearchOptions, SimSearch};
use crate::fs::file_front::{FileChildrenCache, FileChildrenCacheRef};
use crate::io::loading_state::LoadingState;
use crate::widgets::fuzzy_search::helpers::is_subsequence;

// how many file paths should be available for immediate querying "at hand".
// basically a default size of cache for fuzzy file search
const DEFAULT_FILES_PRELOADS: usize = 128 * 1024;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct WrappedRcPath(pub Rc<PathBuf>);

impl Borrow<Path> for WrappedRcPath {
    fn borrow(&self) -> &Path {
        self.0.as_path()
    }
}

impl Borrow<Rc<PathBuf>> for WrappedRcPath {
    fn borrow(&self) -> &Rc<PathBuf> {
        &self.0
    }
}

pub struct InternalState {
    children_cache: HashMap<WrappedRcPath, Rc<RefCell<FileChildrenCache>>>,
    // I need to store some identifier, as search_index.remove requires it. I choose u128 so I can
    // safely not reuse them.
    paths: HashMap<WrappedRcPath, u128>,
    rev_paths: HashMap<u128, WrappedRcPath>,
    pub at_hand_limit: usize, // TODO privatize

    current_idx: u128,
}

impl Default for InternalState {
    fn default() -> Self {
        InternalState {
            children_cache: HashMap::default(),
            paths: HashMap::default(),
            rev_paths: HashMap::default(),
            at_hand_limit: DEFAULT_FILES_PRELOADS,
            current_idx: 1,
        }
    }
}

impl Debug for InternalState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "internal_state: {} paths, {} caches, current_idx = {}", self.paths.len(), self.children_cache.len(), self.current_idx)
    }
}

pub struct FuzzyFileIt<'a> {
    is: &'a InternalState,
    query: String,
    _idx: usize,
}

impl InternalState {
    pub fn get_or_create_cache(&mut self, path: &Rc<PathBuf>) -> FileChildrenCacheRef {
        match self.children_cache.get(path) {
            None => {
                let cache = Rc::new(RefCell::new(FileChildrenCache::default()));
                self.children_cache.insert(WrappedRcPath(path.clone()), cache.clone());
                FileChildrenCacheRef(cache)
            }
            Some(cache) => FileChildrenCacheRef(cache.clone()),
        }
    }

    pub fn get_cache(&self, path: &Path) -> Option<FileChildrenCacheRef> {
        self.children_cache.get(path).map(|f| FileChildrenCacheRef(f.clone()))
    }

    pub fn get_path(&self, path: &Path) -> Option<Rc<PathBuf>> {
        self.paths.get_key_value(path).map(|(p, _)| p.0.clone())
    }

    pub fn remove_path(&mut self, path: &Path) -> bool {
        if !self.paths.contains_key(path) {
            return false;
        }

        let (key, value) = self.paths.get_key_value(path).map(|(a, b)| (a.0.clone(), *b)).unwrap();

        if Rc::strong_count(&key) > 3 {
            warn!("removing path with more than three strong referrers - possible leak?");
        }

        self.paths.remove(path);
        self.rev_paths.remove(&value);
        // self.search_index.delete(&value);

        true
    }

    fn _create_path(&mut self, path: &Path) -> Rc<PathBuf> {
        let rcp = Rc::new(path.to_owned());
        let idx = self.current_idx;
        self.current_idx += 1;

        self.paths.insert(WrappedRcPath(rcp.clone()), idx);
        self.rev_paths.insert(idx, WrappedRcPath(rcp.clone()));
        // rcp.to_str().map(|s| {
        //     let x = s.replace("/", "");
        //     self.search_index.insert(idx, &x);
        // }).unwrap_or_else(|| {
        //     error!("failed to cast path to string, will not be present in index. Absolutely barbaric!");
        // });

        rcp
    }

    pub fn get_or_create_path(&mut self, path: &Path) -> Rc<PathBuf> {
        if let Some((p, _)) = self.paths.get_key_value(path) {
            p.0.clone()
        } else {
            self._create_path(path)
        }
    }

    pub fn fuzzy_files_it(&self, query: String) -> (LoadingState, Box<dyn Iterator<Item=Rc<PathBuf>> + '_>) {
        // TODO this is dumb as fuck, just to prove rest works
        let iter = self.paths.iter().filter(move |item| {
            item.0.0.to_str().map(|s| is_subsequence(s, &query)).unwrap_or_else(|| {
                error!("fuzzy_files_it: path is not a utf-8 str");
                false
            })
        }).map(|item| item.0.0.clone());

        //TODO not implemented properly informing on loading state
        (LoadingState::Complete, Box::new(iter))
    }
}