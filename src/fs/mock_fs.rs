use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use log::{debug, error, warn};
use streaming_iterator::StreamingIterator;
use crate::new_fs::dir_entry::DirEntry;
use crate::new_fs::filesystem_front::FilesystemFront;
use crate::new_fs::fsf_ref::FsfRef;
use crate::new_fs::path::SPath;
use crate::new_fs::read_error::{ListError, ReadError};
use crate::new_fs::write_error::WriteError;

pub struct MockFS {
    root_path : PathBuf,
    // directory -> file -> contents
    all_files : HashMap<PathBuf, HashMap<PathBuf, Vec<u8>>>,
}

impl MockFS {
    pub fn new<T : Into<PathBuf>>(root_path : T) -> Self {
        let mut all_files : HashMap<PathBuf, HashMap<PathBuf, Vec<u8>>> = HashMap::new();
        all_files.insert(PathBuf::new(), HashMap::new());

        MockFS {
            root_path : root_path.into(),
            all_files,
        }
    }

    pub fn with_file<P : AsRef<Path>, B : Into<Vec<u8>>>(mut self, path : P, bytes : B) -> Self {
        self.add_file(path.as_ref(), bytes.into()).unwrap_or_else(
            |_| error!("failed creating file in mockfs"));
        self
    }

    pub fn add_file(&mut self, path : &Path, bytes : Vec<u8>) -> Result<(), ()> {
        let (parent_path, file_name) = Self::split_path(path)?;

        if !self.all_files.contains_key(&parent_path) {
            self.all_files.insert(parent_path.clone(), HashMap::new());
        }

        let mut folder = self.all_files.get_mut(&parent_path).unwrap();

        if let Some(old_val) = folder.insert(PathBuf::from(file_name), bytes) {
            warn!("overwriting file {:?}", path);
        }

        Ok(())
    }
    
    fn split_path(path : &Path) -> Result<(PathBuf, PathBuf), ()> {
        if path.file_name().is_none() {
            error!("no valid filename {:?}", path);
            return Err(());
        }

        let file_name = PathBuf::from(path.file_name().unwrap());
        let parent_path = path.parent().unwrap_or(Path::new("")).to_path_buf();

        Ok((parent_path, file_name))
    }

    pub fn get_file_mut(&mut self, path :&Path) -> Result<&mut Vec<u8>, ReadError> {
        let (parent_path, file_name) = Self::split_path(path).map_err(|_| ReadError::NotAFilePath)?;
        let folder = self.all_files.get_mut(&parent_path).ok_or(ReadError::FileNotFound)?;
        folder.get_mut(&file_name).ok_or(ReadError::FileNotFound)
    }
}

impl Debug for MockFS {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockFilesystem({})", self.root_path.to_string_lossy())
    }
}

impl FilesystemFront for MockFS {
    fn root_path(&self) -> &PathBuf {
        &self.root_path
    }

    fn blocking_read_entire_file(&self, path: &Path) -> Result<Vec<u8>, ReadError> {
        if !self.exists(path) {
            return Err(ReadError::FileNotFound);
        }

        if !self.is_file(path) {
            return Err(ReadError::NotAFilePath);
        }

        let (parent, me)  = Self::split_path(path).unwrap();
        let folder = self.all_files.get(&parent).unwrap();
        Ok(folder.get(&me).unwrap().clone())
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.all_files.contains_key(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        if let Ok( (parent, me) ) = Self::split_path(path) {
            if let Some( folder) = self.all_files.get(&parent) {
                folder.contains_key(&me)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn hash_seed(&self) -> usize {
        2
    }

    fn blocking_list(&self, path: &Path) -> Result<Vec<DirEntry>, ListError> {
        if !self.exists(path) {
            return Err(ListError::PathNotFound);
        }

        if !self.is_dir(path) {
            return Err(ListError::NotADir);
        }

        let mut items : Vec<DirEntry> = Vec::new();

        for key in self.all_files.keys() {
            if let Ok(suffix) = key.strip_prefix(path) {
                if let Some(first_component) = suffix.components().next() {
                    items.push(DirEntry::new(&first_component))
                }
            }
        }

        let folder = self.all_files.get(path).unwrap();

        for item in folder.keys() {
            items.push(DirEntry::new(item.clone()));
        }

        //TODO maybe not needed or even undesirable
        items.sort();

        Ok(items)
    }

    fn exists(&self, path: &Path) -> bool {
        if self.all_files.contains_key(path) {
            return true;
        }

        if let Ok( (parent, me) ) = Self::split_path(path) {
            error!("{:?} {:?}", parent, me);
            if let Some( folder) = self.all_files.get(&parent) {
                folder.contains_key(&me)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn overwrite_with(&self, path: &Path, stream: &dyn StreamingIterator<Item=[u8]>) -> Result<usize, WriteError> {


        todo!()
    }

    fn to_fsf(self) -> FsfRef {
        FsfRef::new(self)
    }
}

// these are purely API tests, like "does it have semantics I like", not "does it work well"
#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::de;
    use crate::new_fs::mock_fs::MockFS;
    use crate::new_fs::filesystem_front::FilesystemFront;
    use crate::new_fs::read_error::ReadError;

    #[test]
    fn make_some_files() {
        let mockfs = MockFS::new("/tmp")
            .with_file("folder1/file1.txt", "some text")
            .with_file("folder2/file2.txt", "some text2");


        assert_eq!(mockfs.is_dir(&Path::new("folder1")), true);
        assert_eq!(mockfs.is_dir(&Path::new("folder2")), true);
        assert_eq!(mockfs.is_dir(&Path::new("folder3")), false);
        assert_eq!(mockfs.is_dir(&Path::new("")), true);

        assert_eq!(mockfs.is_file(&Path::new("folder1/file1.txt")), true);
        assert_eq!(mockfs.is_file(&Path::new("folder2/file2.txt")), true);
        assert_eq!(mockfs.is_file(&Path::new("folder1")), false);
        assert_eq!(mockfs.is_file(&Path::new("folder2")), false);
        assert_eq!(mockfs.is_file(&Path::new("")), false);

        assert_eq!(mockfs.blocking_list(&Path::new("")).unwrap(), vec![de!("folder1"), de!("folder2")]);

        assert_eq!(mockfs.blocking_read_entire_file(&Path::new("")), Err(ReadError::NotAFilePath));
        assert_eq!(mockfs.blocking_read_entire_file(&Path::new("/folder3")), Err(ReadError::FileNotFound));
        assert_eq!(mockfs.blocking_read_entire_file(&Path::new("folder2")), Err(ReadError::NotAFilePath));
        assert_eq!(mockfs.blocking_read_entire_file(&Path::new("folder1/file1.txt")), Ok("some text".as_bytes().to_vec()));
        assert_eq!(mockfs.blocking_read_entire_file(&Path::new("folder1/file3.txt")), Err(ReadError::FileNotFound));
    }
}