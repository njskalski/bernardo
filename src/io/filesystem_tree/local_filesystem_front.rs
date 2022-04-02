use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs::DirEntry;
use std::hash::BuildHasher;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{iter, thread};
use std::io::empty;

use crossbeam_channel::{Receiver, Sender};
use filesystem::{FileSystem, OsFileSystem};
use log::{debug, error, warn};
use ropey::Rope;
use crate::io::filesystem_tree::file_front::{FileChildrenCache, FileChildrenCacheRef, FileFront};

use crate::io::filesystem_tree::filesystem_front::FilesystemFront;
use crate::io::filesystem_tree::fsfref::FsfRef;
use crate::widgets::fuzzy_search::item_provider::Item;

// TODOs:
// - add removing items from cache
// - add inotify support
// - add building trie tree to enable fuzzy search

// how many file paths should be available for immediate querying "at hand".
// basically a default size of cache for fuzzy file search
const DEFAULT_FILES_PRELOADS: usize = 10 * 1024;

#[derive(Debug)]
pub enum SendFile {
    File(PathBuf),
    Directory(PathBuf),
    // TODO add inotify events for filed being removed, renamed etc.
}

#[derive(Debug)]
pub enum FSUpdate {
    DirectoryUpdate {
        full_path: PathBuf,
        entries: Vec<DirEntry>,
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct WrappedRcPath(Rc<PathBuf>);

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

#[derive(Debug)]
struct InternalState {
    children_cache: HashMap<WrappedRcPath, Rc<RefCell<FileChildrenCache>>>,
    paths: HashSet<WrappedRcPath>,
    at_hand_limit: usize,
}

impl InternalState {
    fn get_or_create_cache(&mut self, path: &Rc<PathBuf>) -> FileChildrenCacheRef {
        match self.children_cache.get(path) {
            None => {
                let cache = Rc::new(RefCell::new(FileChildrenCache::default()));
                self.children_cache.insert(WrappedRcPath(path.clone()), cache.clone());
                FileChildrenCacheRef(cache)
            }
            Some(cache) => FileChildrenCacheRef(cache.clone()),
        }
    }

    fn get_cache(&self, path: &Path) -> Option<FileChildrenCacheRef> {
        self.children_cache.get(path).map(|f| FileChildrenCacheRef(f.clone()))
    }
}

#[derive(Debug)]
pub struct LocalFilesystem {
    fs: OsFileSystem,
    root_path: Rc<PathBuf>,

    fs_channel: (Sender<FSUpdate>, Receiver<FSUpdate>),
    tick_channel: (Sender<()>, Receiver<()>),

    internal_state: Rc<RefCell<InternalState>>,
}

impl LocalFilesystem {
    pub fn new(root: PathBuf) -> FsfRef {
        // TODO check it's directory

        let root_cache = Rc::new(RefCell::new(FileChildrenCache {
            complete: false,
            children: vec![],
        }));

        let root_path = Rc::new(root);

        let mut internal_state = InternalState {
            children_cache: HashMap::default(),
            paths: HashSet::default(),
            at_hand_limit: DEFAULT_FILES_PRELOADS,
        };
        internal_state.children_cache.insert(WrappedRcPath(root_path.clone()), root_cache.clone());

        FsfRef(
            Rc::new(Box::new(LocalFilesystem {
                fs: OsFileSystem::new(),
                root_path,
                fs_channel: crossbeam_channel::unbounded::<FSUpdate>(),
                tick_channel: crossbeam_channel::unbounded(),
                internal_state: Rc::new(RefCell::new(internal_state)),
            }))
        )
    }

    pub fn set_at_hand_limit(&self, new_at_hand_limit: usize) {
        self.internal_state.try_borrow_mut().map(|mut is| {
            is.at_hand_limit = new_at_hand_limit;
        }).unwrap_or_else(|e| {
            error!("failed to acquire internal_state: {}", e);
        })
    }

    pub fn with_at_hand_limit(self, new_at_hand_limit: usize) -> Self {
        self.set_at_hand_limit(new_at_hand_limit);
        self
    }

    fn start_fs_refresh(&self, path: &Path) {
        let path = path.to_owned();
        let fs_sender = self.fs_channel.0.clone();
        let tick_sender = self.tick_channel.0.clone();
        let fs = self.fs.clone();

        thread::spawn(move || {
            if !fs.is_dir(&path) {
                warn!("path {:?} is not a dir, ignoring list request", path);
                return;
            }

            // TODO add partitioning

            match fs.read_dir(&path) {
                Err(e) => {
                    error!("failed reading dir {:?}: {}", &path, e);
                    return;
                }
                Ok(rd) => {
                    let mut entries: Vec<DirEntry> = vec![];

                    for de in rd {
                        match de {
                            Err(e) => {
                                error!("failed reading_entry dir in {:?}: {}", &path, e);
                            }
                            Ok(de) => {
                                entries.push(de);
                            }
                        }
                    }

                    fs_sender.send(
                        FSUpdate::DirectoryUpdate {
                            full_path: path,
                            entries,
                        }
                    ).unwrap_or_else(|e| {
                        error!("failed sending dir update for: {}", e);
                    });

                    tick_sender.send(()).unwrap_or_else(|e| {
                        error!("failed sending fs tick: {}", e);
                    });

                    debug!("finished sending dir entries");
                }
            }
        });
    }

    // pub fn get_from_cache(&self, path: &Path) -> Option<FileChildrenCacheRef> {
    //     match self.internal_state.try_borrow() {
    //         Err(e) => {
    //             error!("failed to acquire internal_state: {}", e);
    //             None
    //         }
    //         Ok(is) => {
    //             if let Some(c) = is.children_cache.get(path) {
    //                 Some(c.clone())
    //             } else {
    //                 warn!("cache miss for {:?}", path);
    //                 None
    //             }
    //         }
    //     }
    // }
}

impl FilesystemFront for LocalFilesystem {
    fn get_root_path(&self) -> &Rc<PathBuf> {
        &self.root_path
    }

    fn get_path(&self, path: &Path) -> Option<Rc<PathBuf>> {
        let p: &Path = path;
        if !self.is_within(p) {
            return None;
        }

        match self.internal_state.try_borrow_mut() {
            Ok(mut is) => {
                if let Some(sp) = is.paths.get(path) {
                    Some(sp.0.clone())
                } else {
                    let rc = Rc::new(path.to_owned());
                    is.paths.insert(WrappedRcPath(rc.clone()));
                    Some(rc)
                }
            }
            Err(e) => {
                error!("failed to acquire internal_state: {}", e);
                None
            }
        }
    }

    fn todo_read_file(&self, path: &Path) -> Result<Rope, ()> {
        self.fs.read_file_to_string(path).map(
            |s| Rope::from(s)
        ).map_err(|e|
            error!("failed to read file {:?} : {}", path, e)
        ) // TODO
    }

    // This returns from cache if possible. Triggers update.
    fn get_children_paths(&self, path: &Path) -> (bool, Box<dyn Iterator<Item=Rc<PathBuf>> + '_>) {
        if !self.is_within(path) {
            warn!("requested get_children outside filesystem: {:?}", path);
            return (true, Box::new(iter::empty()));
        }

        if let Ok(is) = self.internal_state.try_borrow() {
            if let Some(cache_ref) = is.get_cache(path) {
                let (complete, children) = cache_ref.get_children();

                debug!("reading from cache {:?} : got {} and {}", path, complete, children.len());

                (complete, Box::new(children.into_iter()))
            } else {
                warn!("no cache for item {:?}", path);

                self.start_fs_refresh(path);

                (true, Box::new(iter::empty()))
            }
        } else {
            error!("failed acquiring internal_state");
            (false, Box::new(iter::empty()))
        }
    }

    fn tick_recv(&self) -> &Receiver<()> {
        &self.tick_channel.1
    }

    fn tick(&self) {
        for msg in self.fs_channel.1.try_iter() {
            // debug!("ticking msg {:?}", msg);
            match msg {
                FSUpdate::DirectoryUpdate { full_path, entries } => {
                    let path = match self.get_path(&full_path) {
                        Some(p) => p,
                        None => { return; }
                    };

                    let mut items: Vec<Rc<PathBuf>> = Vec::new();
                    items.reserve(entries.len());
                    for de in entries.iter() {
                        match de.file_type() {
                            Ok(t) => {
                                self.get_path(&de.path()).map(|item| {
                                    items.push(item);
                                }).unwrap_or_else(|| {
                                    error!("failed to get item for dir_entry: {:?}", de);
                                });
                            }
                            Err(e) => {
                                error!("failed reading file type for dir_entry {:?}: {}", de, e);
                                continue;
                            }
                        }
                    }

                    if let Ok(mut is) = self.internal_state.try_borrow_mut() {
                        let cache = is.get_or_create_cache(&path);
                        cache.0.try_borrow_mut().map(|mut c| {
                            c.complete = true;
                            c.children = items;
                        }).unwrap_or_else(|e| {
                            error!("failed acquiring cache: {}", e);
                        });
                    }
                }
            }
        }
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.fs.is_dir(path)
    }
    fn is_file(&self, path: &Path) -> bool { self.fs.is_file(path) }

    fn is_within(&self, path: &Path) -> bool {
        if !path.starts_with(&*self.root_path) {
            warn!("attempted to open a file from outside of filesystem: {:?}", path);
            false
        } else {
            true
        }
    }

    fn exists(&self, path: &Path) -> bool {
        self.fs.is_dir(path) || self.fs.is_file(path)
    }

    fn todo_save_file_sync(&self, _path: &Path, _bytes: &dyn AsRef<[u8]>) -> Result<(), std::io::Error> {
        // TODO
        // Ok, so fs crate does NOT support appending, which is necessary for streaming etc.
        // Good thing I abstracted over it, will rewrite later.
        //self.fs.overwrite_file(path, &bytes)
        Ok(())
    }
}
