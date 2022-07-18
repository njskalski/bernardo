use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::io::{Error, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossbeam_channel::Receiver;
use ropey::Rope;

use crate::fs::filesystem_front::{FilesystemFront, SomethingToSave};
use crate::fs::read_error::ReadError;
use crate::io::loading_state::LoadingState;

pub enum FakeFileNodeContents {
    Folder(Vec<FakeFileNode>),
    Text(String),
}

pub struct FakeFileNode {
    pub name: PathBuf,
    pub contents : FakeFileNodeContents,
}

impl FakeFileNode {
    pub fn file<P : Into<PathBuf>, T : ToString>(name : P, text : T) -> FakeFileNode {
        FakeFileNode{
            name: name.into(),
            contents: FakeFileNodeContents::Text(text.to_string()),
        }
    }

    pub fn directory<P : Into<PathBuf>>(name : P, items : Vec<FakeFileNode>) -> FakeFileNode {
        FakeFileNode {
            name: name.into(),
            contents : FakeFileNodeContents::Folder(items),
        }
    }

    pub fn directory_from_it<P : Into<PathBuf>, I : Iterator<Item=FakeFileNode>>(name : P, iter : I) -> FakeFileNode {
        FakeFileNode {
            name: name.into(),
            contents : FakeFileNodeContents::Folder(iter.collect()),
        }
    }
}

pub struct FakeFilesystemFront {
    root_path: Arc<PathBuf>,
    root_contents: Vec<FakeFileNode>,

    paths : HashSet<Arc<PathBuf>>,
}

impl FakeFilesystemFront {
    pub fn new(root_path: PathBuf) -> Self {
        let rp = Arc::new(root_path);
        let mut paths = HashSet::new();
        paths.insert(rp.clone());

        FakeFilesystemFront {
            root_path: rp,
            root_contents: Vec::default(),
            paths,
        }
    }
}

impl Debug for FakeFilesystemFront {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FakeFileSystem")
    }
}

impl FilesystemFront for FakeFilesystemFront {

    fn get_root_path(&self) -> &Arc<PathBuf> {
        &self.root_path
    }

    fn get_path(&self, path: &Path) -> Option<Arc<PathBuf>> {
        todo!()
    }

    fn read_entire_file_to_rope(&self, path: &Path) -> Result<Rope, ReadError> {
        todo!()
    }

    fn read_entire_file_bytes(&self, path: &Path) -> Result<Vec<u8>, ReadError> {
        todo!()
    }

    fn get_children_paths(&self, path: &Path) -> (LoadingState, Box<dyn Iterator<Item=Arc<PathBuf>> + '_>) {
        todo!()
    }

    fn tick_recv(&self) -> &Receiver<()> {
        todo!()
    }

    fn tick(&self) {
        todo!()
    }

    fn is_dir(&self, path: &Path) -> bool {
        todo!()
    }

    fn is_file(&self, path: &Path) -> bool {
        todo!()
    }

    fn is_within(&self, path: &Path) -> bool {
        todo!()
    }

    fn exists(&self, path: &Path) -> bool {
        todo!()
    }

    fn fuzzy_file_paths_it(&self, query: String, limit: usize, respect_ignores: bool) -> (LoadingState, Box<dyn Iterator<Item=Arc<PathBuf>> + '_>) {
        todo!()
    }

    fn is_ignored(&self, path: &Path) -> bool {
        todo!()
    }

    fn todo_save_file_sync(&self, path: &Path, bytes: &dyn AsRef<[u8]>) -> Result<(), Error> {
        todo!()
    }

    fn overwrite_file(&self, path: &Path, source: &dyn SomethingToSave) -> Result<(), Error> {
        todo!()
    }
}
#[macro_export]
macro_rules! ffile {
    ( $name:expr, $text: expr) => {
        {
            FakeFileNode::file($name, $text)
        }
    };
}

#[macro_export]
macro_rules! fdir {
    ( $name:expr, $( $it:expr ), *) => {
        {
            let mut items : Vec<FakeFileNode> = Vec::new();
            $(
                items.push($it);
            )*

            FakeFileNode::directory($name, items)
        }
    };
}

// these are purely API tests, like "does it have semantics I like", not "does it work well"
#[cfg(test)]
mod tests {
    use crate::fs::fake_filesystem_front::FakeFileNode;

    #[test]
    fn make_some_files() {
        let f : FakeFileNode = ffile!("some", "text");
        let dir : FakeFileNode = fdir!("dir", f, ffile!("a", "b"), ffile!("x", "d"));
    }
}