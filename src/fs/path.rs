use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use log::warn;
use regex::internal::Input;
use ropey::Rope;
use serde::de::DeserializeOwned;
use streaming_iterator::StreamingIterator;
use syntect::html::IncludeBackground::No;
use crate::fs::dir_entry::DirEntry;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::read_error::{ListError, ReadError};
use crate::fs::write_error::WriteError;

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

    pub fn as_path(&self) -> Option<&Path> {
        match self {
            PathCell::Head(_) => None,
            PathCell::Segment { prev, cell } => Some(cell),
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
        debug_assert!(segment.to_string_lossy().len() > 0);
        debug_assert!(segment.to_string_lossy() != "..");
        debug_assert!(segment.components().count() == 1);
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
        if path.as_ref().to_string_lossy().len() == 0 {
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
        let fsf = self.fsf();
        fsf.blocking_read_entire_file(self)
    }

    pub fn read_entire_file_to_item<T : DeserializeOwned>(&self) -> Result<T, ReadError> {
        let bytes = self.read_entire_file()?;
        ron::de::from_bytes(&bytes).map_err(|e| e.into())
    }

    pub fn read_entire_file_to_string(&self) -> Result<String, ReadError> {
        let bytes = self.read_entire_file()?;
        Ok(String::from_utf8(bytes)?)
    }

    pub fn read_entire_file_to_rope(&self) -> Result<Rope, ReadError> {
        let bytes = self.read_entire_file()?;
        Ok(ropey::Rope::from_reader(&*bytes)?)
    }

    pub fn is_dir(&self) -> bool {
        let fsf = self.fsf();
        fsf.is_dir(self)
    }

    pub fn is_file(&self) -> bool {
        let fsf = self.fsf();
        fsf.is_file(self)
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

    pub fn parent_ref(&self) -> Option<&SPath> {
        match self.0.as_ref() {
            PathCell::Head(_) => None,
            PathCell::Segment { prev, cell } => Some(prev),
        }
    }

    pub fn parent(&self) -> Option<SPath> {
        self.parent_ref().map(|p| p.clone())
    }

    /*
    Returns string representing last component of path
     */
    pub fn file_name_str(&self) -> Option<&str> {
        match self.0.as_ref() {
            PathCell::Head(_) => None,
            PathCell::Segment { prev, cell } => {
                cell.to_str().or_else(|| {
                    warn!("failed casting last item of path {:?}", self);
                    None
                })
            }
        }
    }

    /*
    Returns printable label representing last component for tree/list use case.
    TODO: this should return some non-owned type
     */
    pub fn label(&self) -> Cow<str> {
        match self.0.as_ref() {
            PathCell::Head(fs) => {
                fs.root_path_buf().file_name()
                    .map(|oss|oss.to_string_lossy().into())
                    .unwrap_or_else(||{
                        warn!("failed casting last item of pathbuf. Using hardcoded default.");
                        "<root>".into()
                    })
            },
            PathCell::Segment { prev, cell } => {
                cell.to_string_lossy().into()
            }
        }
    }

    /*
    Returns &Path representing last component of path
     */
    pub fn last_file_name(&self) -> Option<&Path> {
        self.0.as_path()
    }

    /*
    Returns iterator starting in self, and going up the tree until reaching root of filesystem.
     */
    pub fn ancestors_and_self(&self) -> ParentIter {
        ParentIter(self.parent_ref().map(|c| c.clone()))
    }

    /*
    Returns iterator starting in self, and going up the tree until reaching root of filesystem.
     */
    pub fn ancestors_and_self_ref(&self) -> ParentRefIter {
        ParentRefIter(Some(self))
    }

    pub fn is_parent_of(&self, other : &SPath) -> bool {
        let mut iter = other.ancestors_and_self_ref();
        while let Some(parent) = iter.next() {
            if self == parent {
                return true;
            }
        }

        false
    }

    pub fn exists(&self) -> bool {
        // TODO optimise
        let fsf = self.fsf();
        fsf.exists(self)
    }

    pub fn overwrite_with<T : StreamingIterator<Item=[u8]>>(&self, stream : T) -> Result<usize, WriteError> {
        let fsf = self.fsf();
        fsf.overwrite_with(self, &stream)
    }

    pub fn blocking_list(&self) -> Result<Vec<SPath>, ListError> {
        let fsf = self.fsf();
        fsf.blocking_list(self)
    }
}

pub struct ParentIter(Option<SPath>);
pub struct ParentRefIter<'a>(Option<&'a SPath>);

impl<'a> StreamingIterator for ParentRefIter<'a> {
    type Item = SPath;

    fn advance(&mut self) {
        self.0 = self.0.map(|f| f.parent_ref()).flatten();
    }

    fn get(&self) -> Option<&Self::Item> {
        self.0
    }
}

impl Iterator for ParentIter {
    type Item = SPath;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.0.take();
        self.0 = current.as_ref().map(|c| c.parent_ref()).flatten().map(|c| c.clone());
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
    use crate::fs::mock_fs::MockFS;
    use crate::fs::filesystem_front::FilesystemFront;
    use crate::fs::read_error::ReadError;

    #[test]
    fn make_some_files() {}
}