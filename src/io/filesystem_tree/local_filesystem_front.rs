use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{DirEntry, ReadDir};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::Utf8Error;
use std::sync::{Arc, mpsc};
use std::thread;

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

#[derive(Debug)]
pub struct LocalFilesystem {
    fs: OsFileSystem,
    root_node: Rc<FileFront>,

    sender: mpsc::Sender<FSUpdate>,
    receiver: mpsc::Receiver<FSUpdate>,

    internal_state: RefCell<InternalState>,
}

impl LocalFilesystem {
    pub fn new(root: PathBuf) -> Self {
        let (sender, receiver) = mpsc::channel::<FSUpdate>();

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
            sender,
            receiver,
            internal_state: RefCell::new(internal_state),
        }
    }

    fn start_fs_refresh(&self, path: &PathBuf) {
        let path = path.clone();
        let sender = self.sender.clone();
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

                    sender.send(
                        FSUpdate::DirectoryUpdate {
                            full_path: path,
                            entries,
                        }
                    ).map_err(|e| {
                        error!("failed sending dir update for: {}", e);
                    });
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
        todo!()
    }

    fn is_dir(&self, path: &Path) -> bool {
        todo!()
    }

    fn get_children(&self, path: &Path) -> (bool, Box<dyn Iterator<Item=Rc<FileFront>>>) {
        todo!()
    }

    fn todo_expand(&self, path: &Path) {
        todo!()
    }
}
