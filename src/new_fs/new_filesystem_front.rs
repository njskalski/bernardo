use std::fmt::Debug;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::FsfRef;
use crate::new_fs::path::SPath;
use crate::new_fs::read_error::{ListError, ReadError};

// all paths except root_path are RELATIVE to root_path.

pub trait NewFilesystemFront : Debug {
    // Absolute path to root folder. Just for informative reasons.
    fn root_path(&self) -> &PathBuf;

    fn blocking_read_entire_file(&self, path: &Path) -> Result<Vec<u8>, ReadError>;

    /*
    Blocking.
     */
    fn is_dir(&self, path: &Path) -> bool;

    fn hash_seed(&self) -> usize;

    fn list(&self, path: &Path) -> Result<Vec<std::fs::DirEntry>, ListError>;

    fn exists(&self, path: &Path) -> bool;

}