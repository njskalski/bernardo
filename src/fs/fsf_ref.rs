use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs::Metadata;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use log::{debug, error};
use parking_lot::{RwLock, RwLockReadGuard};
use streaming_iterator::StreamingIterator;

use crate::fs::filesystem_front::FilesystemFront;
use crate::fs::path::SPath;
use crate::fs::read_error::{ListError, ReadError};
use crate::fs::write_error::WriteError;

// Chaching should be implemented here or nowhere.

pub struct DirCache {
    vec: Vec<SPath>,
    metadata: Option<Metadata>,
}

impl DirCache {
    pub fn iter(&self) -> impl Iterator<Item = &SPath> {
        self.vec.iter()
    }
}

pub struct ArcIter {
    arc: Arc<DirCache>,
    idx: usize,
}

impl ArcIter {
    pub fn new(arc: Arc<DirCache>) -> Self {
        ArcIter { arc, idx: 0 }
    }
}

impl Iterator for ArcIter {
    type Item = SPath;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.arc.vec.get(self.idx).map(|item| item.clone()) {
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct FsAndCache {
    fs: Box<dyn FilesystemFront + Send + Sync>,
    caches: RwLock<HashMap<SPath, Arc<DirCache>>>,
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
        self.fs.fs.hash_seed() == other.fs.fs.hash_seed() && self.fs.fs.root_path() == other.fs.fs.root_path()
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
            }),
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

    pub fn overwrite_with_stream(
        &self,
        spath: &SPath,
        stream: &mut dyn StreamingIterator<Item = [u8]>,
        must_exist: bool,
    ) -> Result<usize, WriteError> {
        let path = spath.relative_path();
        self.fs.fs.blocking_overwrite_with_stream(&path, stream, must_exist)
    }

    pub fn overwrite_with_str(&self, spath: &SPath, s: &str, must_exist: bool) -> Result<usize, WriteError> {
        let path = spath.relative_path();
        let bytes: Vec<u8> = s.bytes().collect();

        self.fs.fs.blocking_overwrite_with_bytes(&path, &bytes, must_exist)
    }

    pub fn blocking_list(&self, spath: &SPath) -> Result<Arc<DirCache>, ListError> {
        let path = spath.relative_path();
        let metadata = self.fs.fs.metadata(&path).ok();

        // TODO we have "cache invalid never read". Why?
        let mut cache_invalid = false;

        if let Some(system_time) = metadata.as_ref().map(|data| data.modified().ok()).flatten() {
            if let Some(rlock) = self.fs.caches.try_read() {
                if let Some(cached) = rlock.get(spath) {
                    if let Some(cached_time) = cached.metadata.as_ref().map(|m| m.modified().ok()).flatten() {
                        if cached_time == system_time {
                            debug!("cache hit.");

                            // cache is valid
                            return Ok(cached.clone());
                        }

                        if cached_time < system_time {
                            cache_invalid = true;
                        }
                    }
                }
            }
        }

        let items = self.fs.fs.blocking_list(&path)?;
        let mut dir_cache: Vec<SPath> = Vec::with_capacity(items.len());
        for item in items.into_iter() {
            let sp = SPath::append(spath.clone(), item.into_path_buf());
            dir_cache.push(sp);
        }

        dir_cache.sort();

        let dir_cache_arc = Arc::new(DirCache { vec: dir_cache, metadata });

        match self.fs.caches.try_write() {
            Some(mut cache) => {
                cache.insert(spath.clone(), dir_cache_arc.clone());
            }
            None => {
                error!("failed to cache directory list for spath {}", spath)
            }
        }

        Ok(dir_cache_arc)
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
        let mut sp : Option<$crate::fs::path::SPath> = Some($fsf.root());
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
