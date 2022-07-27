use std::borrow::Borrow;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use streaming_iterator::StreamingIterator;
use crate::new_fs::dir_entry::DirEntry;
use crate::new_fs::filesystem_front::FilesystemFront;
use crate::new_fs::path::{PathCell, SPath};
use crate::new_fs::read_error::{ListError, ReadError};
use crate::new_fs::write_error::WriteError;

// Chaching should be implemented here or nowhere.

#[derive(Clone, Debug)]
pub struct FsfRef {
    fs : Arc<Box<dyn FilesystemFront>>,
}

impl PartialEq for FsfRef {
    fn eq(&self, other: &Self) -> bool {
        self.fs.hash_seed() == other.fs.hash_seed() &&
            self.fs.root_path() == other.fs.root_path()
    }
}

impl Eq for FsfRef {}

impl Hash for FsfRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.fs.hash_seed());
        self.fs.root_path().hash(state)
    }
}

impl FsfRef {
    pub fn new<FS : FilesystemFront + 'static>(fs : FS) -> Self {
        FsfRef {
            fs: Arc::new(Box::new(fs) as Box<dyn FilesystemFront>)
        }
    } 
    
    pub fn root(&self) -> SPath {
        SPath::head(self.clone())
    }

    pub fn root_path_buf(&self) -> &PathBuf {
        self.fs.root_path()
    }

    pub fn exists<P: AsRef<Path>>(&self, path : P) -> bool {
        self.fs.as_ref().exists(path.as_ref())
    }

    pub fn descendant_checked<P: AsRef<Path>>(&self, path : P) -> Option<SPath>  {
        let path = path.as_ref();
        if !self.fs.exists(path) {
            return None;
        }

        self.descendant_unchecked(path)
    }

    pub fn descendant_unchecked<P: AsRef<Path>>(&self, path : P) -> Option<SPath> {
        let mut spath = SPath::head(self.clone());
        let mut it = path.as_ref().components();

        while let Some(component) = it.next() {
            let segment = PathBuf::from((&component as &AsRef<Path>).as_ref());
            spath = SPath::append(spath, segment);
        }

        Some(spath)
    }

    pub fn overwrite_with(&self, path : &Path, stream : &dyn StreamingIterator<Item=[u8]>) -> Result<usize, WriteError> {
        self.fs.overwrite_with(path, stream)
    }

    pub fn blocking_list(&self, path: &Path) -> Result<Vec<DirEntry>, ListError> {
        self.fs.blocking_list(path)
    }

    pub fn blocking_read_entire_file(&self, path: &Path) -> Result<Vec<u8>, ReadError> {
        self.fs.blocking_read_entire_file(path)
    }

    pub fn is_dir(&self, path : &Path) -> bool {
        self.fs.is_dir(path)
    }

    pub fn is_file(&self, path : &Path) -> bool {
        self.fs.is_file(path)
    }
}

#[macro_export]
macro_rules! spath{
    ( $fsf:expr $(, $c:expr)* ) => {{
        let mut sp : Option<crate::new_fs::path::SPath> = Some($fsf.root());
        $(
            sp = sp.map(|x| x.descendant_unchecked($c)).flatten();
        )*
        sp
    }};
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::de;
    use crate::new_fs::mock_fs::MockFS;
    use crate::new_fs::filesystem_front::FilesystemFront;
    use crate::new_fs::read_error::ReadError;

    #[test]
    fn spath_macro() {
        let mockfs = MockFS::new("/").to_fsf();
        let sp0 = spath!(mockfs);
        let sp1 = spath!(mockfs, "a");
        let sp2 = spath!(mockfs, "a", "b");
    }
}