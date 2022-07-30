use std::fmt::Debug;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use streaming_iterator::StreamingIterator;

use crate::fs::dir_entry::DirEntry;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::path::SPath;
use crate::fs::read_error::{ListError, ReadError};
use crate::fs::write_error::WriteError;

// all paths except root_path are RELATIVE to root_path.

pub trait FilesystemFront: Debug {
    // Absolute path to root folder. Just for informative reasons.
    fn root_path(&self) -> &PathBuf;

    fn blocking_read_entire_file(&self, path: &Path) -> Result<Vec<u8>, ReadError>;

    /*
    Blocking.
     */
    fn is_dir(&self, path: &Path) -> bool;

    /*
    Blocking

    TODO:
    - define if that's a file, symlink or any
     */
    fn is_file(&self, path: &Path) -> bool;

    fn hash_seed(&self) -> usize;

    fn blocking_list(&self, path: &Path) -> Result<Vec<DirEntry>, ListError>;

    fn exists(&self, path: &Path) -> bool;

    fn blocking_overwrite_with(&self, path: &Path, stream: &mut dyn StreamingIterator<Item=[u8]>) -> Result<usize, WriteError>;

    fn to_fsf(self) -> FsfRef;
}
