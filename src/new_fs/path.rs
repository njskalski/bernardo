use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::de::DeserializeOwned;
use streaming_iterator::StreamingIterator;
use syntect::html::IncludeBackground::No;
use crate::new_fs::fsf_ref::FsfRef;
use crate::new_fs::read_error::ReadError;

// TODO add some invariants.

// SPath points to a file/directory in filesystem.
// We do not allow pieces like "/", ".." or empty path. "Empty" path is a "head" and it points to
//      root of the filesystem, which is expected to be a directory.

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
    Head(FsfRef),
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
    pub fn head(fzf : FsfRef) -> SPath {
        SPath(
            Arc::new(PathCell::Head(fzf))
        )
    }

    pub fn append(prev : SPath, segment : PathBuf) -> SPath {
        debug_assert!(segment.len() > 0);
        debug_assert!(segment != "..");
        SPath(
            Arc::new(PathCell::Segment { prev, cell: segment })
        )
    }

    pub fn fsf(&self) -> &FsfRef {
        match self.0.as_ref() {
            PathCell::Head(fzf) => fzf,
            PathCell::Segment { prev, .. } => prev.fsf(),
        }
    }

    pub fn descendant_checked<P: AsRef<Path>>(&self, path : P) -> Option<SPath>{


        let fzf = self.fsf();
        let full_path = self.relative_path().join(path.as_ref());
        fzf.descendant_checked(full_path)
    }

    // This can still fail if passed:
    //  - empty string
    //  - ".."
    //  - some other nonsensical string that I will add here later.
    pub fn descendant_unchecked<P: AsRef<Path>>(&self, path : P) -> Option<SPath> {
        if path.as_ref().len() == 0 {
            return None;
        }

        let new_cell = path.as_ref().to_path_buf();

        if new_cell == std::path::PathBuf::from("..") {
            return None;
        }

        let spath = SPath::append(self.clone(), new_cell);
        Some(spath)
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

    pub fn last_name(&self) -> Option<&str> {
        match self.0.as_ref() {
            PathCell::Head(_) => Some("<root>"),
            PathCell::Segment { prev, cell } => {
                Some(cell.to_string_lossy().as_ref())
            }
        }
    }

    pub fn ancestors_and_self(&self) -> ParentIter {
        ParentIter(self.parent().map(|c| c.clone()))
    }

    pub fn ancestors_and_self_ref(&self) -> ParentRefIter {
        ParentRefIter(self)
    }

    pub fn is_parent_of(&self, other : &SPath) -> bool {
        while let Some(parent) = other.ancestors_and_self_ref() {
            if self == parent {
                return true;
            }
        }

        false
    }
}

struct ParentIter(Option<SPath>);
struct ParentRefIter<'a>(Option<&'a SPath>);

impl<'a> StreamingIterator for ParentRefIter<'a> {
    type Item = SPath;

    fn advance(&mut self) {
        self.0 = self.0.parent();
    }

    fn get(&self) -> Option<&Self::Item> {
        self.0
    }
}

impl Iterator for ParentIter {
    type Item = SPath;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.0.take();
        self.0 = current.map(|c| c.parent()).flatten().map(|c| c.clone());
        current
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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::de;
    use crate::new_fs::mock_fs::MockFS;
    use crate::new_fs::new_filesystem_front::NewFilesystemFront;
    use crate::new_fs::read_error::ReadError;

    #[test]
    fn make_some_files() {}
}