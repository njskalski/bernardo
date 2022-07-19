use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::fs::DirEntry;
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use log::{debug, error, warn};
use crate::new_fs::new_filesystem_front::NewFilesystemFront;
use crate::new_fs::path::SPath;
use crate::new_fs::read_error::{ListError, ReadError};

pub struct MockFS {
    root_path : PathBuf,
    // directory -> file -> contents
    all_files : HashMap<PathBuf, HashMap<PathBuf, Vec<u8>>>,
}

impl MockFS {
    pub fn new(root_path : PathBuf) -> Self {
        MockFS {
            root_path,
            all_files : HashMap::new(),
        }
    }

    pub fn add_file(&mut self, path : &Path, bytes : Vec<u8>) -> Result<(), ()> {
        let (parent_path, file_name) = Self::split_path(path)?;

        if !self.all_files.contains_key(&parent_path) {
            self.all_files.insert(parent_path.clone(), HashMap::new());
        }

        let mut folder = self.all_files.get_mut(&parent_path).unwrap();

        if let Some(old_val) = folder.insert(PathBuf::from(file_name), bytes) {
            warn!("overwriting file {:?}", path);
        }

        Ok(())
    }

    fn split_path(path : &Path) -> Result<(PathBuf, PathBuf), ()> {
        if path.file_name().is_none() {
            error!("no valid filename {:?}", path);
            return Err(());
        }

        let file_name = PathBuf::from(path.file_name().unwrap());
        let parent_path = path.parent().unwrap_or(Path::new("")).to_path_buf();

        Ok((parent_path, file_name))
    }

    pub fn get_file_mut(&mut self, path :&Path) -> Result<&mut Vec<u8>, ReadError> {
        let (parent_path, file_name) = Self::split_path(path).map_err(|_| ReadError::NotAFilePath)?;
        let folder = self.all_files.get_mut(&parent_path).ok_or(ReadError::FileNotFound)?;
        folder.get_mut(&file_name).ok_or(ReadError::FileNotFound)
    }
}

impl Debug for MockFS {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockFilesystem({})", self.root_path.to_string_lossy())
    }
}

impl NewFilesystemFront for MockFS {
    fn root_path(&self) -> &PathBuf {
        &self.root_path
    }

    fn blocking_read_entire_file(&self, path: &Path) -> Result<Vec<u8>, ReadError> {
        todo!()
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.all_files.contains_key(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        todo!()
    }

    fn hash_seed(&self) -> usize {
        2
    }

    fn list(&self, path: &Path) -> Result<Vec<DirEntry>, ListError> {
        todo!()
    }

    fn exists(&self, path: &Path) -> bool {
        todo!()
    }
}