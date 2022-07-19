use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::fs::DirEntry;
use std::hash::{Hash, Hasher};
use std::io::{Error, Read};
use std::path::{Path, PathBuf};
use filesystem::ReadDir;
use log::{debug, error};
use crate::new_fs::new_filesystem_front::NewFilesystemFront;
use crate::new_fs::path::SPath;
use crate::new_fs::read_error::{ListError, ReadError};

pub struct RealFS {
    root_path : PathBuf,
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

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn hash_seed(&self) -> usize {
        1
    }

    fn list(&self, path : &Path) -> Result<Vec<DirEntry>, ListError> {
        let readdir = std::fs::read_dir(path).map_err(|e| ReadError::UnmappedError(e))?;
        let mut items : Vec<DirEntry> = Vec::new();
        for item in readdir {
            match item {
                Ok(dir_entry) => {
                    items.push(dir_entry)
                }
                Err(e) => {
                    error!("failed read dir because {}", e);
                    Err(ListError::UnmappedError(e))
                }
            }
        }
        Ok(items)
    }

    fn exists(&self, path: &Path) -> bool {
        todo!()
    }
}