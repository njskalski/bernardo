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

impl Hash for NfsfRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fs.hash(state)
    }
}

impl AsRef<Box<dyn FilesystemFront>> for NfsfRef {
    fn as_ref(&self) -> &Box<dyn FilesystemFront> {
        self.fs.as_ref()
    }
}

impl NfsfRef {
    pub fn root(&self) -> SPath {
        SPath::head(self.clone())
    }

    pub fn root_path_buf(&self) -> &PathBuf {
        self.fs.root_path()
    }

    pub fn descendant_checked(&self, path : &Path) -> Option<SPath> {
        if !self.fs.exists(path) {
            return None;
        }

        //TODO can add caching, but not now.

        let mut spath = SPath::head(self.clone());
        let mut it = path.components();

        while let Some(component) = it.next() {
            let segment : PathBuf = component.as_ref().to_owned();
            spath = SPath::append(spath, segment);
        }

        Some(spath)
    }
}