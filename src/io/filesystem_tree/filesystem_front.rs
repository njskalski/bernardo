use std::path::Path;
use std::rc::Rc;

use crossbeam_channel::Receiver;
use ropey::Rope;

use crate::io::filesystem_tree::file_front::FileFront;

pub type FsfRef = Rc<Box<dyn FilesystemFront>>;

pub trait SomethingToSave {
    fn get_bytes(&self) -> Box<dyn Iterator<Item=&u8> + '_>;
}

impl SomethingToSave for Vec<u8> {
    fn get_bytes(&self) -> Box<dyn Iterator<Item=&u8> + '_> {
        Box::new(self.iter())
    }
}

impl SomethingToSave for Rc<String> {
    fn get_bytes(&self) -> Box<dyn Iterator<Item=&u8> + '_> {
        Box::new(self.as_bytes().iter())
    }
}

pub trait FilesystemFront {
    fn get_root(&self) -> Rc<FileFront>;

    // This is a mock method. It should probably return a stream and should probably report errors.
    // One of many "nice to haves" of this editor, outside of scope of MVP, is "large files support",
    // that I want to test with infinite file generator behind an interface here.
    fn todo_read_file(&self, path: &Path) -> Result<Rope, ()>;

    // first argument says if the list is complete.
    // none = true, empty iterator
    fn get_children(&self, path: &Path) -> (bool, Box<dyn Iterator<Item=Rc<FileFront>>>);

    // This schedules refresh of subdirectory, fsf will "tick" once ready to refresh.
    fn todo_expand(&self, path: &Path);

    // this is a channel where it waits for a tick.
    fn tick_recv(&self) -> &Receiver<()>;

    fn tick(&self);

    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;

    fn exists(&self, path: &Path) -> bool;

    //TODO:
    // - backup mechanism (don't loose data on crash)
    // - streaming save
    // - async save
    fn todo_save_file_sync(&self, path: &Path, bytes: &dyn AsRef<[u8]>) -> Result<(), std::io::Error>;
}

