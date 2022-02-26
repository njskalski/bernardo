use std::borrow::Borrow;
use std::cell::{BorrowMutError, RefCell, RefMut};
use std::collections::HashMap;
use std::fs::{DirEntry, ReadDir};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::Utf8Error;
use std::sync::{Arc, mpsc};
use std::thread;

use crossbeam_channel::{Receiver, Sender};
use filesystem::{FileSystem, OsFileSystem};
use log::{debug, error, warn};
use ropey::Rope;

use crate::io::filesystem_tree::file_front::{FileChildrenCache, FileFront, FileType};
use crate::io::filesystem_tree::filesystem_front::FilesystemFront;
use crate::text::buffer_state::BufferState;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

#[derive(Debug)]
pub enum SendFile {
    File(PathBuf),
    Directory(PathBuf),
}

#[derive(Debug)]
pub enum FSUpdate {
    DirectoryUpdate {
        full_path: PathBuf,
        entries: Vec<DirEntry>,
    }
}

#[derive(Debug)]
struct InternalState {
    caches: HashMap<Rc<PathBuf>, Rc<RefCell<FileChildrenCache>>>,
}

impl InternalState {
    fn get_or_create_cache(&mut self, path: &Rc<PathBuf>) -> Rc<RefCell<FileChildrenCache>> {
        match self.caches.get(path) {
            None => {
                let cache = Rc::new(RefCell::new(FileChildrenCache::default()));
                self.caches.insert(path.clone(), cache.clone());
                cache
            }
            Some(cache) => cache.clone(),
        }
    }
}

#[derive(Debug)]
pub struct LocalFilesystem {
    fs: OsFileSystem,
    root_node: Rc<FileFront>,

    fs_channel: (Sender<FSUpdate>, Receiver<FSUpdate>),
    tick_channel: (Sender<()>, Receiver<()>),

    internal_state: RefCell<InternalState>,
}

impl LocalFilesystem {
    pub fn new(root: PathBuf) -> Self {
        // TODO check it's directory

        let root_cache = Rc::new(RefCell::new(FileChildrenCache {
            complete: false,
            children: vec![],
        }));

        let root_path = Rc::new(root);
        let root_node = FileFront::new_directory(root_path.clone(), root_cache.clone());

        let mut internal_state = InternalState {
            caches: HashMap::default(),
        };
        internal_state.caches.insert(root_path.clone(), root_cache.clone());

        LocalFilesystem {
            fs: OsFileSystem::new(),
            root_node: Rc::new(root_node),
            fs_channel: crossbeam_channel::unbounded::<FSUpdate>(),
            tick_channel: crossbeam_channel::unbounded(),
            internal_state: RefCell::new(internal_state),
        }
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
                    ).map_err(|e| {
                        error!("failed sending dir update for: {}", e);
                    });

                    tick_sender.send(()).map_err(|e| {
                        error!("failed sending fs tick: {}", e);
                    });

                    debug!("finished sending dir entries");
                }
            }
        });
    }
}

impl FilesystemFront for LocalFilesystem {
    fn get_root(&self) -> Rc<FileFront> {
        self.root_node.clone()
    }

    fn todo_read_file(&self, path: &Path) -> Result<Rope, ()> {
        self.fs.read_file_to_string(path).map(
            |s| Rope::from(s)
        ).map_err(|e| ()) // TODO
    }

    fn get_children(&self, path: &Path) -> (bool, Box<dyn Iterator<Item=Rc<FileFront>>>) {
        todo!()
    }

    fn todo_expand(&self, path: &Path) {
        self.start_fs_refresh(path)
    }

    fn tick_recv(&self) -> &Receiver<()> {
        &self.tick_channel.1
    }

    fn tick(&self) {
        let mut is = match self.internal_state.try_borrow_mut() {
            Ok(is) => is,
            Err(e) => {
                error!("failed acquiring internal_state: {}", e);
                return;
            }
        };

        for msg in self.fs_channel.1.try_iter() {
            debug!("ticking msg {:?}", msg);
            match msg {
                // TODO now everything is
                FSUpdate::DirectoryUpdate { full_path, entries } => {
                    let path = Rc::new(full_path);

                    let mut items: Vec<Rc<FileFront>> = Vec::new();
                    items.reserve(entries.len());
                    for de in entries.iter() {
                        match de.file_type() {
                            Ok(t) => {
                                // TODO recycle old cache?
                                let child_path = Rc::new(de.path());

                                if t.is_dir() {
                                    let child_cache = is.get_or_create_cache(&child_path);
                                    items.push(Rc::new(FileFront::new_directory(child_path, child_cache)));
                                } else {
                                    items.push(Rc::new(FileFront::new_file(child_path.clone())))
                                }
                            }
                            Err(e) => {
                                error!("failed reading file type for {:?}: {}", de.path(), e);
                                continue
                            }
                        }
                    }

                    let cache = is.get_or_create_cache(&path);
                    cache.try_borrow_mut().map(|mut c| {
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
