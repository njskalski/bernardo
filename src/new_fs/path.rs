use std::path::PathBuf;
use std::sync::Arc;
use crate::FsfRef;

use crate::new_fs::new_filesystem_front::NfsfRef;
use crate::new_fs::read_error::ReadError;

#[derive(Clone, Debug)]
pub enum PathPredecessor {
    FilesystemRoot(NfsfRef),
    SPath(SPath),
}

#[derive(Clone, Debug)]
pub struct PathCell {
    part : PathBuf,
    prev : PathPredecessor,
}

#[derive(Clone, Debug)]
pub struct SPath (pub Arc<PathCell>);

impl PathCell {
    pub fn copy_path(&self) -> PathBuf {
        let mut prefix : PathBuf = match &self.prev {
            PathPredecessor::FilesystemRoot(fsf) => {
                fsf.0.root_path().clone()
            }
            PathPredecessor::SPath(s) => {
                s.0.copy_path()
            }
        };

        prefix.push(&self.part);
        prefix
    }

    pub fn get_fsf(&self) -> &NfsfRef {
        match &self.prev {
            PathPredecessor::FilesystemRoot(fsf) => fsf,
            PathPredecessor::SPath(x) => x.0.get_fsf(),
        }
    }
}

impl Into<PathBuf> for &SPath {
    fn into(self) -> PathBuf {
        self.0.copy_path()
    }
}

impl SPath {
    fn blocking_read_entire_file(&self) -> Result<Vec<u8>, ReadError> {
        let path : PathBuf = self.into();
        let fsf = self.0.get_fsf();
        fsf.0.blocking_read_entire_file(&path)
    }
}