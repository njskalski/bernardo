use std::fmt::Debug;
use std::fs::DirEntry;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/*
Reasons for this thing to exist (use cases in order of importance):
- abstract over filesystem. I will need this for tests, and for remote filesystems.
- inotify support. Refresh support for when filesystem is changed in the background.
- fast queries. We need to execute "fuzzy search" over filenames. This requires precomputing a trie/patricia tree, and updating it on inotify.
- async IO without async runtime. I will test for infinite files support and I want to access huge files over internet.
 */

use crossbeam_channel::Receiver;
use ropey::Rope;

use crate::io::filesystem_tree::file_front::FileFront;
use crate::io::filesystem_tree::LoadingState;


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

/*
Now FilesystemFront does not ever return a FileFront, because for that a FsfRef (Rc<Self>) is needed.
So all methods that return FileFront are in Fsf implementation, and are filesystem agnostic.
 */
pub trait FilesystemFront: Debug {
    fn get_root_path(&self) -> &Rc<PathBuf>;

    fn get_path(&self, path: &Path) -> Option<Rc<PathBuf>>;

    // This is a mock method. It should probably return a stream and should probably report errors.
    // One of many "nice to haves" of this editor, outside of scope of MVP, is "large files support",
    // that I want to test with infinite file generator behind an interface here.
    fn todo_read_file(&self, path: &Path) -> Result<Rope, ()>;

    // first argument says if the list is complete.
    fn get_children_paths(&self, path: &Path) -> (LoadingState, Box<dyn Iterator<Item=Rc<PathBuf>> + '_>);

    // fn ls(&self, path: &Path) -> (bool, Box<dyn Iterator<Item=Rc<PathBuf>> + '_>);

    // This schedules refresh of subdirectory, fsf will "tick" once ready to refresh.
    // fn todo_expand(&self, path: &Path);

    // this is a channel where it waits for a tick.
    fn tick_recv(&self) -> &Receiver<()>;

    fn tick(&self);

    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;

    fn is_within(&self, path: &Path) -> bool;

    fn exists(&self, path: &Path) -> bool;

    //TODO:
    // - backup mechanism (don't loose data on crash)
    // - streaming save
    // - async save
    fn todo_save_file_sync(&self, path: &Path, bytes: &dyn AsRef<[u8]>) -> Result<(), std::io::Error>;
}

