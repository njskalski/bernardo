use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::FsfRef;
use crate::new_fs::nfsf_ref::NfsfRef;

use crate::new_fs::read_error::ReadError;


impl Hash for PathCell {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            // PathPredecessor::FilesystemRoot(f) => state.write_usize(f.0.hash_seed()),
            // PathPredecessor::SPath(s) => s.0.hash(state)
            PathCell::Head(fzf) => fzf.hash(state),
            PathCell::Segment { prev, cell } => {
                cell.hash(state);
                prev.hash(state)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum PathCell {
    Head(NfsfRef),
    Segment{
        prev : SPath,
        cell : PathBuf,
    }
}


#[derive(Clone, Debug)]
pub struct SPath (pub Arc<PathCell>);

impl SPath {
    pub fn head(fzf : NfsfRef) -> SPath {
        SPath(
            Arc::new(PathCell::Head(fzf))
        )
    }

    pub fn append(prev : SPath, segment : PathBuf) -> SPath {
        SPath(
            Arc::new(PathCell::Segment { prev, cell: segment })
        )
    }
}

impl PartialEq<Self> for SPath {
    fn eq(&self, other: &Self) -> bool {
        if self.0.fsf() != other.0.fsf() {
            return false;
        }

        // TODO optimise it
        let path_a = self.relative_path();
        let path_b = self.relative_path();
        path_a == path_b
    }
}

impl Hash for SPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.part.hash(state);
        self.0.prev.hash(state)
    }
}

impl Eq for SPath {}

impl PathCell {
    // fn relative_path(&self) -> PathBuf {
    //     let mut pb : PathBuf = match self {
    //         // PathPredecessor::FilesystemRoot(_) => PathBuf::new(),
    //         // PathPredecessor::SPath(s) => s.0.relative_path(),
    //         PathCell::Head(fzf) => fzf.root_path(),
    //         PathCell::Segment { prev, cell } => {
    //
    //         }
    //     };
    //
    //     pb.push(&self.part);
    //     pb
    // }
    //
    // fn copy_path(&self) -> PathBuf {
    //     let mut prefix : PathBuf = match &self {
    //         PathPredecessor::FilesystemRoot(fsf) => {
    //             fsf.0.root_path().clone()
    //         }
    //         PathPredecessor::SPath(s) => {
    //             s.0.copy_path()
    //         }
    //     };
    //
    //     prefix.push(&self.part);
    //     prefix
    // }
    //
    // fn fsf(&self) -> &NfsfRef {
    //     match &self.prev {
    //         PathPredecessor::FilesystemRoot(fsf) => fsf,
    //         PathPredecessor::SPath(x) => x.0.fsf(),
    //     }
    // }
}

impl Into<PathBuf> for &SPath {
    fn into(self) -> PathBuf {
        self.0.copy_path()
    }
}

impl SPath {
    pub fn blocking_read_entire_file(&self) -> Result<Vec<u8>, ReadError> {
        let path : PathBuf = self.into();
        let fsf = self.0.fsf();
        fsf.0.blocking_read_entire_file(&path)
    }

    pub fn is_dir(&self) -> bool {
        let path : PathBuf = self.into();
        let fsf = self.0.fsf();
        fsf.0.is_dir(&path)
    }

    // returns owned PathBuf relative to FS root.
    pub fn relative_path(&self) -> PathBuf {
        self.0.relative_path()
    }

    pub fn parent(&self) -> Option<&SPath> {
        match self.0.as_ref() {
            PathCell::Head(_) => None,
            PathCell::Segment { prev, cell } => Some(prev),
        }
    }

    pub fn descendant_checked(&self, relative_path: &Path) -> Option<SPath> {
        let fsf = self.0.fsf();


    }
}

