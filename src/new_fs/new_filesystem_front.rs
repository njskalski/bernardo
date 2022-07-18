use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::new_fs::read_error::ReadError;

#[derive(Clone, Debug)]
pub struct NfsfRef(pub Arc<Box<dyn NewFilesystemFront>>);

pub trait NewFilesystemFront : Debug{
    // Absolute path to root folder. Just for informative reasons.
    fn root_path(&self) -> &PathBuf;

    fn blocking_read_entire_file(&self, path : &Path) -> Result<Vec<u8>, ReadError>;
}