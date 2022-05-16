use std::{iter, thread};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::fs::DirEntry;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crossbeam_channel::{Receiver, Sender};
use filesystem::{FileSystem, OsFileSystem};
use log::{debug, error, warn};
use ropey::Rope;

use crate::fs::filesystem_front::{FilesystemFront, ReadError, SomethingToSave};
use crate::fs::fsfref::FsfRef;
use crate::fs::internal_state::InternalState;
use crate::io::loading_state::LoadingState;

// TODOs:
// - add removing items from cache
// - add inotify support

const DEFAULT_MAX_DEPTH: usize = 256;
const GITIGNORE_FILENAME: &'static str = ".gitignore";

#[derive(Debug)]
pub enum FSUpdate {
    DirectoryUpdate {
        full_path: PathBuf,
        entries: Vec<DirEntry>,
    },
    GitignoreFile {
        // gitignore contains path.
        gitignore: ignore::gitignore::Gitignore,
    },
}

#[derive(Debug)]
pub enum FSRequest {
    RefreshDir {
        full_path: PathBuf,
        max_depth: usize,
    },
    PauseIndexing,
    UnpauseIndexing,
    KillIndexer,
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

        let mut internal_state = InternalState::new(root.clone());
        let root_path = internal_state.get_or_create_path(&root);
        internal_state.get_or_create_cache(&root_path);

        let fs = LocalFilesystem {
            fs: OsFileSystem::new(),
            root_path,
            fs_request_channel: crossbeam_channel::unbounded::<FSRequest>(),
            fs_channel: crossbeam_channel::unbounded::<FSUpdate>(),
            tick_channel: crossbeam_channel::bounded(1),
            internal_state: Rc::new(RefCell::new(internal_state)),
        };

        fs.start_indexing_thread();

        FsfRef(Rc::new(Box::new(fs)))
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

    fn request_refresh(&self, path: &Path) -> bool {
        self.fs_request_channel.0.try_send(
            FSRequest::RefreshDir { full_path: path.to_owned(), max_depth: 1 }
        ).map_err(|e| {
            error!("request_refresh: failed requesting dir refresh for {:?}: {}", path, e);
        }).is_ok()
    }

    /*
    Indexer is basically a thread that does following:
    1) if we have a request to refresh a dir, this is what has highest priority
    2) otherwise it indexes whatever is in the queue
    3) if queue is empty, it waits for the dir
     */
    fn start_indexing_thread(&self) {
        // TODO add:
        // - max message size
        // - skip refresh if not updated
        // - ignore parameters
        // - make sure only one is running?
        // - add resuming on error?


        let fs = self.fs.clone();
        let root_path = self.root_path.as_path().to_owned();
        let response_channel = self.fs_channel.0.clone();
        let request_channel = self.fs_request_channel.1.clone();
        let tick_channel = self.tick_channel.0.clone();

        thread::spawn(move || {
            // what, how deep
            let mut pipe: VecDeque<(PathBuf, usize)> = VecDeque::new();
            pipe.push_back((root_path, DEFAULT_MAX_DEPTH));

            'indexing_loop:
            loop {
                let mut paused = false;

                // I moved message processing here, because I call it twice (once after "try_recv" and once after "recv").
                // returns whether the loop should be broken or not.
                let mut process_request = |req: FSRequest, pipe: &mut VecDeque<(PathBuf, usize)>| -> bool {
                    match req {
                        FSRequest::RefreshDir {
                            full_path,
                            max_depth,
                        } => {
                            if !paused {
                                debug!("pushing {:?} in front of request queue of len {}", &full_path, pipe.len());
                                pipe.push_front((full_path, max_depth));
                            } else {
                                debug!("ignoring request {:?} because paused", &full_path);
                            }
                            false
                        }
                        FSRequest::PauseIndexing => {
                            debug!("pausing indexing.");
                            pipe.clear();
                            paused = true;
                            false
                        }
                        FSRequest::UnpauseIndexing => {
                            debug!("unpausing indexing.");
                            paused = false;
                            false
                        }
                        FSRequest::KillIndexer => {
                            debug!("killing indexer thread");
                            true
                        }
                    }
                };

                // first we drain request_channel, to put interrupting high-priority requests in the front.
                while let Ok(req) = request_channel.try_recv() {
                    let should_break_loop = process_request(req, &mut pipe);
                    if should_break_loop {
                        break 'indexing_loop;
                    }
                }

                // this is the only place where this thread "sleeps". It does it only when pipe is empty.
                if pipe.is_empty() {
                    loop {
                        match request_channel.recv() {
                            Ok(req) => {
                                let should_break_loop = process_request(req, &mut pipe);
                                if should_break_loop {
                                    break 'indexing_loop;
                                }
                                if !pipe.is_empty() {
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("failed retrieving indexing request: {}", e);
                                break 'indexing_loop;
                            }
                        }
                    }
                }

                // if we got here, pipe is not empty. But that doesn't stop me from being paranoid.
                let (what, how_deep): (PathBuf, usize) = match pipe.pop_front() {
                    None => {
                        error!("pipe should not be empty here, but it was!");
                        break 'indexing_loop;
                    }
                    Some((head, how_deep)) => (head, how_deep),
                };

                // debug!("pipe len {}, processing {:?}, depth: {:?}", pipe.len(), &what, how_deep);

                let mut gitignore_found = false;

                let mut children: Vec<DirEntry> = vec![];
                match fs.read_dir(&what) {
                    Ok(read_dir) => {
                        'items_loop:
                        for item in read_dir {
                            match item {
                                Ok(dir_entry) => {
                                    match dir_entry.file_type() {
                                        Ok(ft) => {
                                            if ft.is_dir() {
                                                if how_deep > 0 {
                                                    pipe.push_back((dir_entry.path(),
                                                                    how_deep - 1)
                                                    );
                                                }
                                            }
                                            if ft.is_file() && &dir_entry.file_name() == GITIGNORE_FILENAME {
                                                gitignore_found = true
                                            }

                                            children.push(dir_entry);
                                        }
                                        Err(e) => {
                                            error!("failed retrieving file_type of {:?}: {}", dir_entry ,e);
                                            break 'items_loop;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("failed getting dir_entry: {}", e);
                                    break 'items_loop;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("failed reading dir {:?}: {}", &what, e);
                    }
                }

                // this could be lower, but then I would have to call .clone() on what *always*, as opposed to *rarely*
                if gitignore_found {
                    let gitignore_path = what.clone().join(GITIGNORE_FILENAME);
                    let (gitignore, error_op) = ignore::gitignore::Gitignore::new(&gitignore_path);
                    match error_op {
                        Some(err) => {
                            error!("got errors parsing gitignore at {:?}: {}", gitignore_path, err);
                        }
                        None => {
                            response_channel.try_send(FSUpdate::GitignoreFile {
                                gitignore,
                            }).map_err(|e| {
                                error!("failed sending gitignore file: {}", e);
                            });
                        }
                    }
                }

                let msg = FSUpdate::DirectoryUpdate {
                    full_path: what,
                    entries: children,
                };

                match response_channel.send(msg) {
                    Err(e) => {
                        debug!("failed sending response: {}", e);
                        break 'indexing_loop;
                    }
                    _ => {}
                };


                tick_channel.try_send(());
            }

            debug!("indexer thread finishes");
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

    fn read_entire_file_to_rope(&self, path: &Path) -> Result<Rope, ReadError> {
        let mut file = std::fs::File::open(path).map_err(|ioe| ReadError::IoError(ioe))?;
        let mut buf: Vec<u8> = Vec::default();
        file.read_to_end(&mut buf);
        let s = std::str::from_utf8(&buf).map_err(|ue| ReadError::Utf8Error(ue))?;

        Ok(Rope::from_str(s))
    }

    fn read_entire_file_bytes(&self, path: &Path) -> Result<Vec<u8>, ReadError> {
        let mut file = std::fs::File::open(path).map_err(|ioe| ReadError::IoError(ioe))?;
        let mut buf: Vec<u8> = Vec::default();
        file.read_to_end(&mut buf);

        Ok(buf)
    }


    // This returns from cache if possible. Triggers update.
    fn get_children_paths(&self, path: &Path) -> (LoadingState, Box<dyn Iterator<Item=Rc<PathBuf>> + '_>) {
        if !self.is_within(path) {
            warn!("requested get_children outside fs: {:?}", path);
            return (LoadingState::Complete, Box::new(iter::empty()));
        }

        if let Ok(is) = self.internal_state.try_borrow_mut() {
            if let Some(cache_ref) = is.get_cache(path) {
                let (loading_state, children) = cache_ref.get_children();

                if loading_state != LoadingState::InProgress && loading_state != LoadingState::Complete {
                    debug!("requesting refresh of {:?} because it's {:?}", path, loading_state);
                    self.request_refresh(path);
                }

                // debug!("reading from cache {:?} : got {} and {}", path, loading_state, children.len());
                (loading_state, Box::new(children.into_iter()))
            } else {
                warn!("no cache for item {:?}", path);
                self.request_refresh(path);
                (LoadingState::InProgress, Box::new(iter::empty()))
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
            Err(..) => {
                error!("tick: failed acquiring fs lock");
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
                            Ok(..) => {
                                let de_path = is.get_or_create_path(&de.path());
                                items.push(de_path);
                            }
                            Err(e) => {
                                error!("failed reading file type for dir_entry {:?}: {}", de, e);
                                continue;
                            }
                        }
                    }

                    // debug!("ticking on is: {:?}", is);

                    let cache = is.get_or_create_cache(&path);
                    cache.0.try_borrow_mut().map(|mut c| {
                        c.loading_state = LoadingState::Complete;
                        c.children = items;
                    }).unwrap_or_else(|e| {
                        error!("failed acquiring cache: {}", e);
                    });
                }
                FSUpdate::GitignoreFile { gitignore } => {
                    is.set_gitignore(gitignore);
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
            warn!("attempted to open a file from outside of fs: {:?}", path);
            false
        } else {
            true
        }
    }

    fn exists(&self, path: &Path) -> bool {
        self.fs.is_dir(path) || self.fs.is_file(path)
    }

    fn fuzzy_file_paths_it(&self, query: String, limit: usize, respect_ignores: bool) -> (LoadingState, Box<dyn Iterator<Item=Rc<PathBuf>> + '_>) {
        self.internal_state.try_borrow().map(|isref| {
            let (loading_state, iter) = isref.fuzzy_files_it(query);

            let items: Vec<Rc<PathBuf>> = if respect_ignores {
                iter.filter(|item| isref.is_ignored(item) == false).take(limit).map(|f| f.clone()).collect()
            } else {
                iter.take(limit).map(|f| f.clone()).collect()
            };

            (loading_state, Box::new(items.into_iter()) as Box<dyn Iterator<Item=Rc<PathBuf>>>)
        }).unwrap_or_else(|e| {
            error!("failed acquiring lock: {}", e);
            (LoadingState::Error, Box::new(iter::empty()))
        })
    }

    fn is_ignored(&self, path: &Path) -> bool {
        self.internal_state.try_borrow_mut().map(|isref| {
            isref.is_ignored(path)
        }).unwrap_or_else(|e| {
            error!("failed acquiring lock: {}", e);
            false
        })
    }

    fn todo_save_file_sync(&self, _path: &Path, _bytes: &dyn AsRef<[u8]>) -> Result<(), std::io::Error> {
        // TODO
        // Ok, so fs crate does NOT support appending, which is necessary for streaming etc.
        // Good thing I abstracted over it, will rewrite later.
        //self.fs.overwrite_file(path, &bytes)
        Ok(())
    }

    // TODO add is_within test?
    fn overwrite_file(&self, path: &Path, source: &dyn SomethingToSave) -> Result<(), std::io::Error> {
        let mut file = std::fs::File::create(path)?;

        for slice in source.get_slices() {
            file.write_all(slice)?;
        };

        Ok(())
    }
}