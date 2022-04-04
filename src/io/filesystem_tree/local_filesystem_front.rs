use std::borrow::Borrow;
use std::cell::{BorrowMutError, RefCell, RefMut};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::DirEntry;
use std::hash::BuildHasher;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{iter, thread};
use std::fmt::{Debug, Formatter};
use std::io::empty;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam_channel::{Receiver, Sender};
use filesystem::{FileSystem, OsFileSystem};
use log::{debug, error, warn};
use ropey::Rope;
use simsearch::SimSearch;
use crate::io::filesystem_tree::file_front::{FileChildrenCache, FileChildrenCacheRef, FileFront};

use crate::io::filesystem_tree::filesystem_front::FilesystemFront;
use crate::io::filesystem_tree::fsfref::FsfRef;
use crate::io::filesystem_tree::internal_state::{InternalState, WrappedRcPath};
use crate::io::filesystem_tree::LoadingState;
use crate::widgets::fuzzy_search::item_provider::Item;

// TODOs:
// - add removing items from cache
// - add inotify support
// - add building trie tree to enable fuzzy search


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

#[derive(Debug)]
pub enum FSRequest {
    RefreshDir {
        full_path: PathBuf
    }
}

#[derive(Debug)]
pub struct LocalFilesystem {
    fs: OsFileSystem,
    root_path: Rc<PathBuf>,

    fs_request_channel: (Sender<FSRequest>, Receiver<FSRequest>),
    fs_channel: (Sender<FSUpdate>, Receiver<FSUpdate>),
    tick_channel: (Sender<()>, Receiver<()>),

    internal_state: Rc<RefCell<InternalState>>,
}

impl LocalFilesystem {
    pub fn new(root: PathBuf) -> FsfRef {
        // TODO check it's directory

        let mut internal_state = InternalState::default();
        let root_path = internal_state.get_or_create_path(&root);
        internal_state.get_or_create_cache(&root_path);

        FsfRef(
            Rc::new(Box::new(LocalFilesystem {
                fs: OsFileSystem::new(),
                root_path,
                fs_request_channel: crossbeam_channel::unbounded::<FSRequest>(),
                fs_channel: crossbeam_channel::unbounded::<FSUpdate>(),
                tick_channel: crossbeam_channel::bounded(1),
                internal_state: Rc::new(RefCell::new(internal_state)),
            }))
        )
    }

    pub fn set_at_hand_limit(&self, new_at_hand_limit: usize) {
        self.internal_state.try_borrow_mut().map(|mut is| {
            is.at_hand_limit = new_at_hand_limit;
        }).unwrap_or_else(|e| {
            error!("set_at_hand_limit: failed to acquire internal_state: {}", e);
        })
    }

    pub fn with_at_hand_limit(self, new_at_hand_limit: usize) -> Self {
        self.set_at_hand_limit(new_at_hand_limit);
        self
    }

    fn start_dir_refresh(&self, path: &Path) {
        let fs = self.fs.clone();

        if !fs.is_dir(&path) {
            warn!("path {:?} is not a dir, ignoring list request", path);
            return;
        }

        let fs_sender = self.fs_channel.0.clone();
        let tick_sender = self.tick_channel.0.clone();

        let path = path.to_owned();

        thread::spawn(move || {
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

    fn internal_index_root(&self, cancellation_flag: Option<Arc<AtomicBool>>) {
        let fs = self.fs.clone();
        let root_path = self.root_path.as_path().to_owned();
        let channel = self.fs_channel.0.clone();
        let tick_channel = self.tick_channel.0.clone();

        // TODO add:
        // - max message size
        // - max depth?
        // - ignore parameters

        thread::spawn(move || {
            let mut pipe: VecDeque<PathBuf> = VecDeque::new();
            pipe.push_back(root_path);

            while !pipe.is_empty() {
                let head = pipe.pop_front().unwrap();
                debug!("pipe len {}, processing {:?}", pipe.len(), &head);

                let mut children: Vec<DirEntry> = vec![];

                match fs.read_dir(&head) {
                    Ok(read_dir) => {
                        for item in read_dir {
                            match item {
                                Ok(dir_entry) => {
                                    match dir_entry.file_type() {
                                        Ok(ft) => {
                                            if ft.is_dir() {
                                                pipe.push_back(dir_entry.path());
                                            }
                                            children.push(dir_entry);
                                        }
                                        Err(e) => {
                                            error!("failed retrieving file_type of {:?}: {}", dir_entry ,e);
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("failed getting dir_entry: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("failed reading dir {:?}: {}", head, e);
                        break;
                    }
                }

                let msg = FSUpdate::DirectoryUpdate {
                    full_path: head,
                    entries: children,
                };

                if channel.send(msg).is_err() {
                    break;
                }
                tick_channel.try_send(());

                if cancellation_flag.as_ref().map(|c| c.load(Ordering::Relaxed)).unwrap_or(false) {
                    break;
                }
            }

            debug!("indexing done");
        });
    }
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

        // TODO I am not sure of this - get_or_create or just get?
        match self.internal_state.try_borrow_mut() {
            Ok(mut is) => {
                Some(is.get_or_create_path(path))
            }
            Err(e) => {
                error!("get_path: failed to acquire internal_state: {}", e);
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
    fn get_children_paths(&self, path: &Path) -> (LoadingState, Box<dyn Iterator<Item=Rc<PathBuf>> + '_>) {
        if !self.is_within(path) {
            warn!("requested get_children outside filesystem: {:?}", path);
            return (LoadingState::Complete, Box::new(iter::empty()));
        }

        let maybe_refresh = |mut is: RefMut<InternalState>| -> LoadingState {
            let pathrc = match is.get_path(path) {
                Some(p) => p,
                None => {
                    error!("refreshing unknown path {:?}? Dropping.", path);
                    return LoadingState::Error;
                }
            };

            let cache = is.get_or_create_cache(&pathrc);

            let loading_state = cache.get_loading_state();
            match loading_state {
                LoadingState::InProgress | LoadingState::Complete | LoadingState::Error => {}
                LoadingState::NotStarted => {
                    cache.set_loading_state(LoadingState::InProgress);
                    self.start_dir_refresh(path);
                }
            }

            loading_state
        };

        if let Ok(mut is) = self.internal_state.try_borrow_mut() {
            if let Some(cache_ref) = is.get_cache(path) {
                let (mut loading_state, children) = cache_ref.get_children();

                if loading_state != LoadingState::InProgress {
                    loading_state = maybe_refresh(is);
                }
                debug!("reading from cache {:?} : got {} and {}", path, loading_state, children.len());

                (loading_state, Box::new(children.into_iter()))
            } else {
                warn!("no cache for item {:?}", path);
                let loading_state = maybe_refresh(is);
                (loading_state, Box::new(iter::empty()))
            }
        } else {
            error!("get_children_paths: failed acquiring internal_state");
            (LoadingState::Error, Box::new(iter::empty()))
        }
    }

    fn tick_recv(&self) -> &Receiver<()> {
        &self.tick_channel.1
    }

    fn tick(&self) {
        let mut is = match self.internal_state.try_borrow_mut() {
            Ok(is) => is,
            Err(e) => {
                error!("tick: failed acquiring filesystem lock");
                return;
            }
        };

        for msg in self.fs_channel.1.try_iter() {
            // debug!("ticking msg {:?}", msg);
            match msg {
                FSUpdate::DirectoryUpdate { full_path, entries } => {
                    let path = is.get_or_create_path(&full_path);

                    let mut items: Vec<Rc<PathBuf>> = Vec::new();
                    items.reserve(entries.len());
                    for de in entries.iter() {
                        match de.file_type() {
                            Ok(t) => {
                                let de_path = is.get_or_create_path(&de.path());
                                items.push(de_path);
                            }
                            Err(e) => {
                                error!("failed reading file type for dir_entry {:?}: {}", de, e);
                                continue;
                            }
                        }
                    }

                    debug!("ticking on is: {:?}", is);

                    let cache = is.get_or_create_cache(&path);
                    cache.0.try_borrow_mut().map(|mut c| {
                        c.loading_state = LoadingState::Complete;
                        c.children = items;
                    }).unwrap_or_else(|e| {
                        error!("failed acquiring cache: {}", e);
                    });
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

    fn index_root(&self, cancellation_flag: Option<Arc<AtomicBool>>) {
        self.internal_index_root(cancellation_flag)
    }
}
