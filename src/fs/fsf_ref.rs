use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use log::error;
use streaming_iterator::StreamingIterator;

use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::path::SPath;
use crate::fs::read_error::{ListError, ReadError};
use crate::fs::write_error::WriteError;

// Chaching should be implemented here or nowhere.

pub struct DirCache {
    vec: Arc<Vec<SPath>>,
}

pub struct FsAndCache {
    fs: Box<dyn FilesystemFront + Send + Sync>,
    caches: RwLock<HashMap<SPath, DirCache>>,
}

#[derive(Clone)]
pub struct FsfRef {
    fs: Arc<FsAndCache>,
}

impl Debug for FsfRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FsfRef{:?}", self.fs.fs)
    }
}

impl PartialEq for FsfRef {
    fn eq(&self, other: &Self) -> bool {
        self.fs.fs.hash_seed() == other.fs.fs.hash_seed() &&
            self.fs.fs.root_path() == other.fs.fs.root_path()
    }
}

impl Eq for FsfRef {}

impl Hash for FsfRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.fs.fs.hash_seed());
        self.fs.fs.root_path().hash(state)
    }
}

impl FsfRef {
    pub fn new<FS: FilesystemFront + Sync + Send + 'static>(fs: FS) -> Self {
        let fsf = FsfRef {
            fs: Arc::new(FsAndCache {
                fs: Box::new(fs) as Box<dyn FilesystemFront + Sync + Send>,
                caches: RwLock::new(Default::default()),
            })
        };

        fsf
    }

    pub fn root(&self) -> SPath {
        SPath::head(self.clone())
    }

    pub fn root_path_buf(&self) -> &PathBuf {
        self.fs.fs.root_path()
    }

    pub fn exists(&self, path: &SPath) -> bool {
        let path = path.relative_path();
        self.fs.fs.exists(&path)
    }

    pub fn descendant_checked<P: AsRef<Path>>(&self, path: P) -> Option<SPath> {
        let path = path.as_ref();
        if !self.fs.fs.exists(path) {
            return None;
        }

        self.descendant_unchecked(path)
    }

    pub fn descendant_unchecked<P: AsRef<Path>>(&self, path: P) -> Option<SPath> {
        let mut spath = SPath::head(self.clone());
        let mut it = path.as_ref().components();

        while let Some(component) = it.next() {
            let segment = PathBuf::from((&component as &dyn AsRef<Path>).as_ref());
            spath = SPath::append(spath, segment);
        }

        Some(spath)
    }

    pub fn overwrite_with_stream(&self, spath: &SPath, stream: &mut dyn StreamingIterator<Item=[u8]>) -> Result<usize, WriteError> {
        let path = spath.relative_path();
        self.fs.fs.blocking_overwrite_with_stream(&path, stream)
    }

    pub fn overwrite_with_str(&self, spath: &SPath, s: &str) -> Result<usize, WriteError> {
        let path = spath.relative_path();
        self.fs.fs.blocking_overwrite_with_str(&path, s)
    }

    pub fn blocking_list(&self, spath: &SPath) -> Result<Arc<Vec<SPath>>, ListError> {
        // TODO unwrap - not necessary
        if let Some(cache) = self.fs.caches.try_read().unwrap().get(spath) {
            return Ok(cache.vec.clone());
        }

        let path = spath.relative_path();
        let items = self.fs.fs.blocking_list(&path)?;

        let mut dir_cache: Vec<SPath> = Vec::with_capacity(items.len());
        for item in items.into_iter() {
            let sp = SPath::append(spath.clone(), item.into_path_buf());
            dir_cache.push(sp);
        }

        let arc = Arc::new(dir_cache);

        match self.fs.caches.try_write() {
            Ok(mut cache) => {
                cache.insert(spath.clone(), DirCache {
                    vec: arc.clone(),
                });
            }
            Err(e) => {
                error!("failed writing cache, because {:?}", e);
            }
        }

        Ok(arc)
    }

    pub fn blocking_read_entire_file(&self, spath: &SPath) -> Result<Vec<u8>, ReadError> {
        let path = spath.relative_path();
        self.fs.fs.blocking_read_entire_file(&path)
    }

    pub fn is_dir(&self, spath: &SPath) -> bool {
        let path = spath.relative_path();
        self.fs.fs.is_dir(&path)
    }

    pub fn is_file(&self, spath: &SPath) -> bool {
        let path = spath.relative_path();
        self.fs.fs.is_file(&path)
    }
}

#[macro_export]
macro_rules! spath {
    ( $fsf:expr $(, $c:expr)* ) => {{
        #[allow(unused_mut)]
        let mut sp : Option<crate::fs::path::SPath> = Some($fsf.root());
        $(
            sp = sp.map(|s| s.descendant_unchecked($c)).flatten();
        )*
        sp
    }};
}

#[cfg(test)]
mod tests {
    use crate::fs::filesystem_front::FilesystemFront;
    use crate::fs::mock_fs::MockFS;

    #[test]
    fn spath_macro() {
        let mockfs = MockFS::new("/").to_fsf();
        let _sp0 = spath!(mockfs);
        let _sp1 = spath!(mockfs, "a");
        let _sp2 = spath!(mockfs, "a", "b");
    }
}