use std::fmt::{Debug, Formatter};
use std::fs::Metadata;
use std::io;
use std::io::{Error, Write};
use std::path::{Path, PathBuf};

use log::{error, info, warn};
use streaming_iterator::StreamingIterator;

use crate::fs::dir_entry::DirEntry;
use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::read_error::{ListError, ReadError};
use crate::fs::write_error::WriteError;

pub struct RealFS {
    root_path: PathBuf,
}

impl RealFS {
    pub fn new(root_path: PathBuf) -> io::Result<RealFS> {
        let root_path = if root_path.is_absolute() {
            root_path
        } else {
            root_path.canonicalize()?
        };

        Ok(RealFS { root_path })
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

    fn blocking_list(&self, path: &Path) -> Result<Vec<DirEntry>, ListError> {
        let fullpath = self.root_path.join(path);
        let readdir = match std::fs::read_dir(&fullpath) {
            Ok(r) => r,
            Err(e) => {
                error!(target: "fsf", "failed to read_dir {:?} because {}", &fullpath, e);
                return Err(ListError::UnmappedError(e.to_string()));
            }
        };

        let mut items: Vec<DirEntry> = Vec::new();
        for item in readdir {
            match item {
                Ok(dir_entry) => match dir_entry.path().file_name() {
                    Some(file_name) => {
                        items.push(DirEntry::new(file_name));
                    }
                    None => {
                        warn!("received dir_entry {:?} that does not have file_name, ignoring.", dir_entry);
                    }
                },
                Err(e) => {
                    error!("failed read dir because {}", e);
                    return Err(e.into());
                }
            }
        }
        Ok(items)
    }

    fn metadata(&self, path: &Path) -> Result<Metadata, ReadError> {
        let fullpath: PathBuf = self.root_path.join(path);

        match std::fs::metadata(&fullpath) {
            Ok(meta) => Ok(meta),
            Err(e) => {
                info!(target: "fsf", "failed reading metadata for {:?}, because {}", fullpath, e);
                Err(ReadError::UnmappedError(e.to_string()))
            }
        }
    }

    fn file_size(&self, path: &Path) -> Result<u64, ReadError> {
        self.metadata(path).map(|meta| meta.len())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn blocking_overwrite_with_stream(
        &self,
        path: &Path,
        stream: &mut dyn StreamingIterator<Item = [u8]>,
        must_exist: bool,
    ) -> Result<usize, WriteError> {
        if must_exist && path.exists() {
            return Err(WriteError::FileNotFound);
        }

        let mut file = std::fs::File::create(path)?;

        let mut bytes_written: usize = 0;
        while let Some(bytes) = stream.next() {
            let num_bytes = match file.write(bytes) {
                Ok(num_bytes) => {
                    num_bytes
                },
                Err(e) => {
                    error!("failed to write to file {} because {}", path.to_string_lossy(), e);
                    return Err(WriteError::UnmappedError(e.to_string()));
                }
            };


            if num_bytes != bytes.len() {
                error!("unexpected number of bytes written");
                break;
            }
            bytes_written += num_bytes;
        }

        file.flush()?;
        Ok(bytes_written)
    }

    fn blocking_overwrite_with_bytes(&self, path: &Path, s: &[u8], must_exist: bool) -> Result<usize, WriteError> {
        if must_exist && path.exists() {
            return Err(WriteError::FileNotFound);
        }

        std::fs::write(path, s)?;
        Ok(s.len())
    }

    fn to_fsf(self) -> FsfRef {
        FsfRef::new(self)
    }
}
