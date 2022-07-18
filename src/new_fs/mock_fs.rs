use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use crate::new_fs::new_filesystem_front::NewFilesystemFront;
use crate::new_fs::path::SPath;
use crate::new_fs::read_error::ReadError;

pub struct MockFS {
    root_path : PathBuf,
    all_files : HashMap<PathBuf, String>
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
        if let Some(text) = &self.all_files.get(path) {
            Ok(text.as_bytes().to_vec())
        } else {
            Err(ReadError::FileNotFound)
        }
    }
}