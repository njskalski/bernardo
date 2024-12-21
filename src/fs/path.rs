use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use log::{debug, error, warn};
use ropey::Rope;
use serde::de::DeserializeOwned;
use serde::Serialize;
use streaming_iterator::StreamingIterator;
use url::Url;

use crate::cursor::cursor::{Cursor, Selection};
use crate::fs::fsf_iter::RecursiveFsIter;
use crate::fs::fsf_ref::{ArcIter, FsfRef};
use crate::fs::read_error::{ListError, ReadError};
use crate::fs::search_error::SearchError;
use crate::fs::write_error::{WriteError, WriteOrSerError};
use crate::primitives::common_query::CommonQuery;
use crate::primitives::printable::Printable;
use crate::primitives::symbol_usage::SymbolUsage;
use crate::promise::streaming_promise::StreamingPromise;
use crate::promise::streaming_promise_impl::WrappedMspcReceiver;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathCell {
    Head(FsfRef),
    Segment { prev: SPath, cell: PathBuf },
}

impl PathCell {
    pub fn relative_path(&self) -> PathBuf {
        match &self {
            PathCell::Head(_) => PathBuf::new(),
            PathCell::Segment { prev, cell } => {
                let mut head = prev.relative_path();
                head = head.join(cell);

                // head.join(cell)
                head
            }
        }
    }

    pub fn as_path(&self) -> Option<&Path> {
        match &self {
            PathCell::Head(_) => None,
            PathCell::Segment { prev: _, cell } => Some(cell),
        }
    }

    pub fn is_head(&self) -> bool {
        match &self {
            PathCell::Head(_) => true,
            PathCell::Segment { .. } => false,
        }
    }
}

#[derive(Clone)]
pub struct SPath(pub Arc<PathCell>);

impl SPath {
    pub fn head(fzf: FsfRef) -> SPath {
        SPath(Arc::new(PathCell::Head(fzf)))
    }

    pub fn append<P: AsRef<Path>>(prev: SPath, segment: P) -> SPath {
        let cell = segment.as_ref().to_path_buf();
        debug_assert!(cell.to_string_lossy().len() > 0);
        debug_assert!(cell.to_string_lossy() != "..");
        debug_assert!(cell.components().count() == 1);

        SPath(Arc::new(PathCell::Segment { prev, cell }))
    }

    pub fn fsf(&self) -> &FsfRef {
        match self.0.as_ref() {
            PathCell::Head(fzf) => fzf,
            PathCell::Segment { prev, .. } => prev.fsf(),
        }
    }

    pub fn descendant_checked<P: AsRef<Path>>(&self, path: P) -> Option<SPath> {
        let fzf = self.fsf();
        let full_path = self.relative_path().join(path.as_ref());
        fzf.descendant_checked(full_path)
    }

    // This can still fail if passed:
    //  - empty string
    //  - ".."
    //  - some other nonsensical string that I will add here later.
    pub fn descendant_unchecked<P: AsRef<Path>>(&self, path: P) -> Option<SPath> {
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

    pub fn read_entire_file_to_item<T: DeserializeOwned>(&self) -> Result<T, ReadError> {
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

    pub fn is_hidden(&self) -> bool {
        match self.file_name_str() {
            Some(fname) => fname.starts_with('.'),
            None => false,
        }
    }

    // returns owned PathBuf relative to FS root.
    pub fn relative_path(&self) -> PathBuf {
        self.0.relative_path()
    }

    pub fn absolute_path(&self) -> PathBuf {
        let path = self.relative_path();
        let root_path = self.fsf().root_path_buf().clone();
        root_path.join(path)
    }

    pub fn parent_ref(&self) -> Option<&SPath> {
        match self.0.as_ref() {
            PathCell::Head(_) => None,
            PathCell::Segment { prev, cell: _ } => Some(prev),
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
            PathCell::Segment { prev: _, cell } => cell.to_str().or_else(|| {
                warn!("failed casting last item of path {:?}", self);
                None
            }),
        }
    }

    /*
    Returns printable label representing last component for tree/list use case.
     */
    pub fn label(&self) -> Cow<str> {
        match self.0.as_ref() {
            PathCell::Head(fs) => fs
                .root_path_buf()
                .file_name()
                .map(|oss| oss.to_string_lossy().into())
                .unwrap_or_else(|| {
                    warn!("failed casting last item of pathbuf. Using hardcoded default.");
                    "<root>".into()
                }),
            PathCell::Segment { prev: _, cell } => cell.to_string_lossy().into(),
        }
    }

    /*
    Returns &Path representing last component of path. Empty for Filesystem Root.
     */
    pub fn last_file_name(&self) -> Option<&Path> {
        self.0.as_path()
    }

    /*
    Returns iterator starting in self, and going up the tree until reaching root of filesystem.
     */
    pub fn ancestors_and_self(&self) -> ParentIter {
        ParentIter(Some(self.clone()))
    }

    /*
    Returns iterator starting in self, and going up the tree until reaching root of filesystem.
     */
    pub fn ancestors_and_self_ref(&self) -> ParentRefIter {
        ParentRefIter::new(Some(self))
    }

    pub fn is_parent_of(&self, other: &SPath) -> bool {
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

    pub fn overwrite_with_stream(&self, stream: &mut dyn StreamingIterator<Item = [u8]>, must_exist: bool) -> Result<usize, WriteError> {
        let fsf = self.fsf();
        fsf.overwrite_with_stream(self, stream, must_exist)
    }

    pub fn overwrite_with_str<T: AsRef<str>>(&self, s: T, must_exist: bool) -> Result<usize, WriteError> {
        let fsf = self.fsf();
        let ss = s.as_ref();
        fsf.overwrite_with_str(self, ss, must_exist)
    }

    pub fn overwrite_with_ron<T: Serialize>(&self, item: &T, must_exist: bool) -> Result<usize, WriteOrSerError> {
        let ron_item = ron::ser::to_string_pretty::<T>(item, ron::ser::PrettyConfig::default())?;
        self.overwrite_with_str(&ron_item, must_exist).map_err(|e| e.into())
    }

    pub fn blocking_list(&self) -> Result<impl Iterator<Item = SPath> + '_, ListError> {
        let fsf = self.fsf();
        fsf.blocking_list(self).map(|item| ArcIter::new(item))
    }

    // TODO add error?
    pub fn to_url(&self) -> Result<Url, ()> {
        let path = self.absolute_path();
        let url = url::Url::from_file_path(&path);
        if url.is_err() {
            error!("failed casting spath [{}] to url", self);
        }

        url
    }

    pub fn recursive_iter(&self) -> RecursiveFsIter {
        RecursiveFsIter::new(self.clone())
    }

    // TODO this is "muscle" part of editor, perhaps it's worth to move it to a separate file?
    // TODO I would prefer this algorithm to work with BFS as opposed to DFS.
    // TODO add REGEX
    pub fn start_full_text_search(
        &self,
        query: CommonQuery,
        _ignore_git: bool,
    ) -> Result<Box<dyn StreamingPromise<SymbolUsage>>, SearchError> {
        let simple_query = match query {
            CommonQuery::Epsilon => {
                return Err(SearchError::UnsupporedQueryType {
                    details: "empty query not allowed in full text search",
                })
            }
            CommonQuery::String(s) => s,
            CommonQuery::Fuzzy(_) => {
                return Err(SearchError::UnsupporedQueryType {
                    details: "fuzzy query not allowed in full text search",
                })
            }
            CommonQuery::Regex(_) => {
                return Err(SearchError::UnsupporedQueryType {
                    details: "regex query (currently) not allowed in full text search",
                })
            }
        };

        if simple_query.is_empty() {
            return Err(SearchError::MalformedQuery {
                details: "query cannot be empty",
            });
        }

        let (sender, receiver) = crossbeam_channel::unbounded::<SymbolUsage>();

        let root = self.clone();
        std::thread::spawn(move || {
            let mut iter = root.recursive_iter();
            'main_loop: for item in iter {
                // error!("item {}", &item);

                if !item.is_file() {
                    continue;
                }

                match item.read_entire_file_to_string() {
                    Err(e) => {
                        error!("failed reading file {} because {}, continuing.", &item, e);
                    }
                    Ok(string) => {
                        error!("file {}, contents length {}", item, string.len());
                        for hit in string.match_indices(&simple_query) {
                            if hit.1.is_empty() {
                                error!("malformed selection, skipping result at {}", hit.0);
                                continue;
                            }

                            let symbol_usage = SymbolUsage {
                                path: item.clone(),
                                range: Cursor::new(hit.0).with_selection(Selection::new(hit.0, hit.0 + hit.1.graphemes().count())),
                            };

                            match sender.send(symbol_usage) {
                                Ok(_) => {}
                                Err(_) => {
                                    debug!(
                                        "Stopping search for '{}' in '{}' and descendants - channel closed.",
                                        simple_query, root
                                    );
                                    break 'main_loop;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(WrappedMspcReceiver::new(receiver).boxed())
    }
}

pub struct ParentIter(Option<SPath>);

/*
First field is "what is going to be returned next".
Second field says "did initial advance already happen or not", because streaming iterators call
    advance before get, which lead to skipping first element. Field exists only to ignore fist call
    to "advance".
 */
pub struct ParentRefIter<'a> {
    next: Option<&'a SPath>,
    first_advance_happened: bool,
}

impl<'a> ParentRefIter<'a> {
    pub fn new(first: Option<&'a SPath>) -> ParentRefIter<'a> {
        ParentRefIter {
            next: first,
            first_advance_happened: false,
        }
    }
}

impl<'a> StreamingIterator for ParentRefIter<'a> {
    type Item = SPath;

    fn advance(&mut self) {
        if !self.first_advance_happened {
            self.first_advance_happened = true;
            return;
        }

        self.next = self.next.map(|f| f.parent_ref()).flatten();
    }

    fn get(&self) -> Option<&Self::Item> {
        self.next
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

        let path_a = self.relative_path();
        let path_b = other.relative_path();
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

impl PartialOrd<Self> for SPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 == other.0 {
            return Some(Ordering::Equal);
        }

        // probably in after profiling I will decide to optimise here
        self.absolute_path().partial_cmp(&other.absolute_path())
    }
}

impl Ord for SPath {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 == other.0 {
            return Ordering::Equal;
        }

        // probably in after profiling I will decide to optimise here
        self.absolute_path().cmp(&other.absolute_path())
    }
}

#[cfg(test)]
mod tests {
    use streaming_iterator::StreamingIterator;

    use crate::fs::filesystem_front::FilesystemFront;
    use crate::fs::mock_fs::MockFS;
    use crate::spath;

    #[test]
    fn eq() {
        let m1 = MockFS::new("/tmp").to_fsf();
        let m2 = MockFS::new("/tmp2").to_fsf();

        assert_eq!(spath!(m1, "a", "b").unwrap(), spath!(m1, "a", "b").unwrap());
        assert_ne!(spath!(m1, "a", "b").unwrap(), spath!(m1, "a", "c").unwrap());
        assert_ne!(spath!(m1).unwrap(), spath!(m2).unwrap());
        assert_ne!(spath!(m1, "a", "b").unwrap(), spath!(m2, "a", "b").unwrap());
    }

    #[test]
    fn parent_ref() {
        let mockfs = MockFS::new("/tmp").to_fsf();
        let sp = spath!(mockfs, "folder1", "folder2", "file1.txt").unwrap();

        assert_eq!(sp.parent_ref().unwrap(), &spath!(mockfs, "folder1", "folder2").unwrap());
        assert_eq!(sp.parent_ref().unwrap().parent_ref().unwrap(), &spath!(mockfs, "folder1").unwrap());
        assert_eq!(
            sp.parent_ref().unwrap().parent_ref().unwrap().parent_ref().unwrap(),
            &spath!(mockfs).unwrap()
        );
    }

    #[test]
    fn parent_and_self_ref_it() {
        let mockfs = MockFS::new("/tmp").to_fsf();
        let sp = spath!(mockfs, "folder1", "folder2", "folder3", "file1.txt").unwrap();

        let mut it = sp.ancestors_and_self_ref();
        assert_eq!(it.next(), Some(&sp));
        assert_eq!(it.next(), Some(&spath!(mockfs, "folder1", "folder2", "folder3").unwrap()));
        assert_eq!(it.next(), Some(&spath!(mockfs, "folder1", "folder2").unwrap()));
        assert_eq!(it.next(), Some(&spath!(mockfs, "folder1").unwrap()));
        assert_eq!(it.next(), Some(&spath!(mockfs).unwrap()));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn parent() {
        let mockfs = MockFS::new("/tmp").to_fsf();
        let sp = spath!(mockfs, "folder1", "folder2", "file1.txt").unwrap();

        assert_eq!(sp, sp);

        let sparent1 = sp.parent().unwrap();
        assert_eq!(sparent1, spath!(mockfs, "folder1", "folder2").unwrap());
        let sparent2 = sparent1.parent().unwrap();
        assert_eq!(sparent2, spath!(mockfs, "folder1").unwrap());
        let sparent3 = sparent2.parent().unwrap();
        assert_eq!(sparent3, spath!(mockfs).unwrap());
        assert_eq!(sparent3.parent(), None);
    }

    #[test]
    fn parent_and_self_it() {
        let mockfs = MockFS::new("/tmp").to_fsf();
        let sp = spath!(mockfs, "folder1", "folder2", "folder3", "file1.txt").unwrap();

        let mut it = sp.ancestors_and_self();
        assert_eq!(it.next(), Some(sp));
        assert_eq!(it.next(), spath!(mockfs, "folder1", "folder2", "folder3"));
        assert_eq!(it.next(), spath!(mockfs, "folder1", "folder2"));
        assert_eq!(it.next(), spath!(mockfs, "folder1"));
        assert_eq!(it.next(), spath!(mockfs));
        assert_eq!(it.next(), None);
    }
}
