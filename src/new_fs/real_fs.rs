use std::fmt::{Debug, Formatter};
use std::io::{Error, Read};
use std::path::{Path, PathBuf};
use crate::new_fs::new_filesystem_front::NewFilesystemFront;
use crate::new_fs::read_error::ReadError;

pub struct RealFS {
    root_path : PathBuf
}

impl Debug for RealFS {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Filesystem({})", self.root_path.to_string_lossy())
    }
}

impl NewFilesystemFront for RealFS {
    fn root_path(&self) -> &PathBuf {
        &self.root_path
    }

    fn blocking_read_entire_file(&self, path: &Path) -> Result<Vec<u8>, ReadError> {
        std::fs::read(path).map_err(|e| ReadError::UnmappedError(e))
    }
}