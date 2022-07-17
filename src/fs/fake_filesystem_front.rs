use std::fmt::{Debug, Formatter};
use std::io::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossbeam_channel::Receiver;
use ropey::Rope;

use crate::fs::filesystem_front::{FilesystemFront, SomethingToSave};
use crate::fs::read_error::ReadError;
use crate::io::loading_state::LoadingState;

pub enum FakeFileNodeContents {
    Folder(Vec<FakeFileNode>),

}

pub struct FakeFileNode {
    pub name: PathBuf,
    pub children: Vec<FakeFileNode>,
}

pub struct FakeFilesystemFront {
    root_path: Arc<PathBuf>,
    root_contents: Vec<FakeFileNode>,
}

impl FakeFilesystemFront {
    pub fn new(root_path: PathBuf) -> Self {
        FakeFilesystemFront {
            root_path: Arc::new(root_path),
            root_contents: Vec::default(),
        }
    }

    pub fn add_items<I: Iterator<Item=FakeFileNode>>(&mut self, items: I) {}
}

impl Debug for FakeFilesystemFront {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FakeFileSystem")
    }
}

impl FilesystemFront for FakeFilesystemFront {
    fn get_root_path(&self) -> &Arc<PathBuf> {
        &self.root_path
    }

    fn get_path(&self, path: &Path) -> Option<Arc<PathBuf>> {
        todo!()
    }

    fn read_entire_file_to_rope(&self, path: &Path) -> Result<Rope, ReadError> {
        todo!()
    }

    fn read_entire_file_bytes(&self, path: &Path) -> Result<Vec<u8>, ReadError> {
        todo!()
    }

    fn get_children_paths(&self, path: &Path) -> (LoadingState, Box<dyn Iterator<Item=Arc<PathBuf>> + '_>) {
        todo!()
    }

    fn tick_recv(&self) -> &Receiver<()> {
        todo!()
    }

    fn tick(&self) {
        todo!()
    }

    fn is_dir(&self, path: &Path) -> bool {
        todo!()
    }

    fn is_file(&self, path: &Path) -> bool {
        todo!()
    }

    fn is_within(&self, path: &Path) -> bool {
        todo!()
    }

    fn exists(&self, path: &Path) -> bool {
        todo!()
    }

    fn fuzzy_file_paths_it(&self, query: String, limit: usize, respect_ignores: bool) -> (LoadingState, Box<dyn Iterator<Item=Arc<PathBuf>> + '_>) {
        todo!()
    }

    fn is_ignored(&self, path: &Path) -> bool {
        todo!()
    }

    fn todo_save_file_sync(&self, path: &Path, bytes: &dyn AsRef<[u8]>) -> Result<(), Error> {
        todo!()
    }

    fn overwrite_file(&self, path: &Path, source: &dyn SomethingToSave) -> Result<(), Error> {
        todo!()
    }
}