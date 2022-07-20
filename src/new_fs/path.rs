use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::de::DeserializeOwned;
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

impl PathCell {
    pub fn relative_path(&self) -> PathBuf {
        match self {
            PathCell::Head(_) => PathBuf::new(),
            PathCell::Segment { prev, cell } => {
                let mut head = prev.relative_path();
                head.join(cell)
            }
        }
    }
}


#[derive(Clone)]
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

    pub fn fsf(&self) -> &NfsfRef {
        match self.0.as_ref() {
            PathCell::Head(fzf) => fzf,
            PathCell::Segment { prev, .. } => prev.fsf(),
        }
    }

    pub fn descendant_checked<P: AsRef<Path>>(&self, path : P) -> Option<SPath>{
        let fzf = self.fsf();
        fzf.descendant_checked(path.as_ref())
    }

    pub fn read_entire_file(&self) -> Result<Vec<u8>, ReadError> {
        let path : PathBuf = self.relative_path();
        let fsf = self.fsf();
        fsf.fs.blocking_read_entire_file(&path)
    }

    pub fn read_entire_file_to_item<T : DeserializeOwned>(&self) -> Result<T, ReadError> {
        let bytes = self.read_entire_file()?;
        ron::de::from_bytes(&bytes).map_err(|e| e.into())
    }

    pub fn is_dir(&self) -> bool {
        let path : PathBuf = self.relative_path();
        let fsf = self.fsf();
        fsf.fs.is_dir(&path)
    }

    pub fn is_file(&self) -> bool {
        let path : PathBuf = self.relative_path();
        let fsf = self.fsf();
        fsf.fs.is_file(&path)
    }

    // returns owned PathBuf relative to FS root.
    pub fn relative_path(&self) -> PathBuf {
        self.0.relative_path()
    }

    pub fn absolute_path(&self) -> PathBuf {
        let path= self.relative_path();
        let root_path = self.fsf().root_path_buf().clone();
        root_path.join(path)
    }

    pub fn parent(&self) -> Option<&SPath> {
        match self.0.as_ref() {
            PathCell::Head(_) => None,
            PathCell::Segment { prev, cell } => Some(prev),
        }
    }
}

impl PartialEq<Self> for SPath {
    fn eq(&self, other: &Self) -> bool {
        if *self.fsf() != *other.fsf() {
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
        self.0.as_ref().hash(state)
    }
}

impl Eq for SPath {}

impl Display for SPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let path = self.relative_path();
        write!(f, "{}", path.to_string_lossy())
    }
}

impl Debug for SPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let path = self.relative_path();
        write!(f, "{}", path.to_string_lossy())
    }
}