use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DirEntry {
    file_name: PathBuf,
}

impl DirEntry {
    pub fn new<P: Into<PathBuf>>(file_name: P) -> DirEntry {
        DirEntry {
            file_name: file_name.into(),
        }
    }

    pub fn into_path_buf(self) -> PathBuf {
        self.file_name
    }

    pub fn into_path(&self) -> &Path {
        &self.file_name
    }
}

#[macro_export]
macro_rules! de {
    ( $name:expr ) => {{
        crate::fs::dir_entry::DirEntry::new($name)
    }};
}
