use std::fmt::Debug;
use std::path::{Path, PathBuf};

use streaming_iterator::StreamingIterator;

use crate::fs::dir_entry::DirEntry;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::read_error::{ListError, ReadError};
use crate::fs::write_error::WriteError;

// all paths except root_path are RELATIVE to root_path.

pub trait FilesystemFront: Debug + Send + Sync {
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

    fn blocking_overwrite_with_stream(
        &self,
        path: &Path,
        stream: &mut dyn StreamingIterator<Item = [u8]>,
        must_exist: bool,
    ) -> Result<usize, WriteError>;

    fn blocking_overwrite_with_bytes(&self, path: &Path, s: &[u8], must_exist: bool) -> Result<usize, WriteError>;

    fn to_fsf(self) -> FsfRef;
}
