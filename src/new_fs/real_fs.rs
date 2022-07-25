use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{Error, Read};
use std::path::{Path, PathBuf};
use filesystem::ReadDir;
use log::{debug, error};
use streaming_iterator::StreamingIterator;
use crate::new_fs::dir_entry::DirEntry;
use crate::new_fs::new_filesystem_front::NewFilesystemFront;
use crate::new_fs::path::SPath;
use crate::new_fs::read_error::{ListError, ReadError};
use crate::new_fs::write_error::WriteError;

pub struct RealFS {
    root_path : PathBuf,
}

impl RealFS {
    pub fn new(root_path : PathBuf) -> RealFS {
        RealFS {
            root_path
        }
    }
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
        let full_path = self.root_path.join(path);
        std::fs::read(&full_path).map_err(|e| e.into())
    }

    fn is_dir(&self, path: &Path) -> bool {
        let full_path = self.root_path.join(path);
        full_path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        let full_path = self.root_path.join(path);
        full_path.is_file()
    }

    fn hash_seed(&self) -> usize {
        1
    }

    fn list(&self, path : &Path) -> Result<Vec<DirEntry>, ListError> {
        let full_path = self.root_path.join(path);
        let readdir = std::fs::read_dir(&full_path)?;
        let mut items : Vec<DirEntry> = Vec::new();
        for item in readdir {
            match item {
                Ok(dir_entry) => {
                    items.push(DirEntry::new(dir_entry.path()))
                }
                Err(e) => {
                    error!("failed read dir because {}", e);
                    return Err(e.into())
                }
            }
        }
        Ok(items)
    }

    fn exists(&self, path: &Path) -> bool {
        todo!()
    }

    fn overwrite_with(&self, path: &Path, stream: &dyn StreamingIterator<Item=[u8]>) -> Result<usize, WriteError> {
        todo!()
    }
}
