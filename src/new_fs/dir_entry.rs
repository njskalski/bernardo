use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DirEntry {
    file_name : PathBuf,
}

impl DirEntry {
    pub fn new<P : Into<PathBuf>>(file_name: P) -> DirEntry {
        DirEntry {
            file_name : file_name.into()
        }
    }
}

#[macro_export]
macro_rules! de {
    ( $name:expr ) => {
        {
            crate::new_fs::dir_entry::DirEntry::new($name)
        }
    };
}
