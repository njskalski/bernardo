use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::fs::filesystem_front::FilesystemFront;
use crate::new_fs::new_filesystem_front::NewFilesystemFront;
use crate::new_fs::path::{PathCell, SPath};

// Chaching should be implemented here or nowhere.

#[derive(Clone, Debug)]
pub struct NfsfRef{
    pub fs : Arc<Box<dyn NewFilesystemFront>>,
}

impl PartialEq for NfsfRef {
    fn eq(&self, other: &Self) -> bool {
        self.fs.hash_seed() == other.fs.hash_seed() &&
            self.fs.root_path() == other.fs.root_path()
    }
}

impl Eq for NfsfRef {}

impl Hash for NfsfRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.fs.hash_seed());
        self.fs.root_path().hash(state)
    }
}

impl NfsfRef {
    pub fn root(&self) -> SPath {
        SPath::head(self.clone())
    }

    pub fn root_path_buf(&self) -> &PathBuf {
        self.fs.root_path()
    }

    pub fn descendant_checked<P: AsRef<Path>>(&self, path : P) -> Option<SPath>  {
        let path = path.as_ref();
        if !self.fs.exists(path) {
            return None;
        }

        //TODO can add caching, but not now.

        let mut spath = SPath::head(self.clone());
        let mut it = path.components();

        while let Some(component) = it.next() {
            let segment = PathBuf::from((&component as &AsRef<Path>).as_ref());
            spath = SPath::append(spath, segment);
        }

        Some(spath)
    }
}