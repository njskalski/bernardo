use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::fs::DirEntry;
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use crate::new_fs::new_filesystem_front::NewFilesystemFront;
use crate::new_fs::path::SPath;
use crate::new_fs::read_error::{ListError, ReadError};

pub struct MockFS {
    root_path : PathBuf,
    // directory -> file, contents
    all_files : HashMap<PathBuf, (PathBuf, String)>,
}

impl MockFS {
    pub fn new(root_path : PathBuf) -> Self {
        MockFS {
            root_path,
            all_files : HashMap::new(),
        }
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
        let parent = path.parent().ok_or(ReadError::FileNotFound)?;
        if let Some(text) = &self.all_files.get(parent) {
            Ok(text.as_bytes().to_vec())
        } else {
            Err(ReadError::FileNotFound)
        }
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.all_files.contains_key(path)
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