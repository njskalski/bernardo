use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{Error, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use filesystem::ReadDir;
use log::{debug, error, warn};
use streaming_iterator::StreamingIterator;

use crate::fs::dir_entry::DirEntry;
use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::path::SPath;
use crate::fs::read_error::{ListError, ReadError};
use crate::fs::write_error::WriteError;

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

impl FilesystemFront for RealFS {
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

    fn blocking_list(&self, path : &Path) -> Result<Vec<DirEntry>, ListError> {
        let full_path = self.root_path.join(path);
        let readdir = std::fs::read_dir(&full_path)?;
        let mut items : Vec<DirEntry> = Vec::new();
        for item in readdir {
            match item {
                Ok(dir_entry) => {
                    match dir_entry.path().file_name() {
                        Some(file_name) => {
                            items.push(DirEntry::new(file_name));
                        }
                        None => {
                            warn!("received dir_entry {:?} that does not have file_name, ignoring.", dir_entry);
                        }
                    }
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
        path.exists()
    }

    fn blocking_overwrite_with_stream(&self, path: &Path, stream: &mut dyn StreamingIterator<Item=[u8]>) -> Result<usize, WriteError> {
        let mut file = std::fs::File::open(path)?;

        let mut bytes_written: usize = 0;
        while let Some(bytes) = stream.next() {
            let num_bytes = file.write(bytes)?;
            if num_bytes != bytes.len() {
                error!("unexpected number of bytes written");
                break;
            }
        }

        file.flush()?;
        Ok(bytes_written)
    }

    fn blocking_overwrite_with_str(&self, path: &Path, s: &str) -> Result<usize, WriteError> {
        std::fs::write(path, s)?;
        Ok(s.len())
    }

    fn to_fsf(self) -> FsfRef {
        FsfRef::new(self)
    }
}