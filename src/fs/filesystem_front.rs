use std::fmt::{Debug, Display, Formatter};
use std::{io, iter};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::Utf8Error;
use std::sync::Arc;

/*
Reasons for this thing to exist (use cases in order of importance):
- abstract over fs. I will need this for tests, and for remote filesystems.
- inotify support. Refresh support for when fs is changed in the background.
- fast queries. We need to execute "fuzzy search" over filenames. This requires precomputing a trie/patricia tree, and updating it on inotify.
- async IO without async runtime. I will test for infinite files support and I want to access huge files over internet.
 */

use crossbeam_channel::Receiver;
use ropey::Rope;

use crate::io::loading_state::LoadingState;

#[derive(Debug)]
pub enum ReadError {
    IoError(io::Error),
    Utf8Error(Utf8Error),
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ReadError {}

pub trait SomethingToSave {
    fn get_slices(&self) -> Box<dyn Iterator<Item=&[u8]> + '_>;
}

impl SomethingToSave for Vec<u8> {
    fn get_slices(&self) -> Box<dyn Iterator<Item=&[u8]> + '_> {
        Box::new(
            iter::once(
                self.as_slice()
            )
        )
    }
}

impl SomethingToSave for &str {
    fn get_slices(&self) -> Box<dyn Iterator<Item=&[u8]> + '_> {
        Box::new(
            iter::once(
                self.as_bytes()
            )
        )
    }
}

impl SomethingToSave for Rc<String> {
    fn get_slices(&self) -> Box<dyn Iterator<Item=&[u8]> + '_> {
        Box::new(
            iter::once(
                self.as_bytes()
            )
        )
    }
}

/*
Now FilesystemFront does not ever return a FileFront, because for that a FsfRef (Rc<Self>) is needed.
So all methods that return FileFront are in Fsf implementation, and are fs agnostic.
 */
pub trait FilesystemFront: Debug {
    fn get_root_path(&self) -> &Arc<PathBuf>;

    /*
    Converts path to Rc<PathBuf>, creating it if necessary.
    Fails ONLY if the given path is outside root.
     */
    fn get_path(&self, path: &Path) -> Option<Arc<PathBuf>>;

    fn read_entire_file_to_rope(&self, path: &Path) -> Result<Rope, ReadError>;

    fn read_entire_file_bytes(&self, path: &Path) -> Result<Vec<u8>, ReadError>;

    // One of many "nice to haves" of this editor, outside of scope of MVP, is "large files support",
    // that I want to test with infinite file generator behind an interface here.

    // first argument says if the list is complete.
    fn get_children_paths(&self, path: &Path) -> (LoadingState, Box<dyn Iterator<Item=Arc<PathBuf>> + '_>);

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

    // returns files that satisfy query (query is a substring of file name)
    // TODO would be great to not pass the limit ahead, but until I figure out how to wrap a Ref into an iterator, I don't know how.
    // TODO now the fuzzy search actually slows everything down a lot, because it's retriggered each keystroke. I should cache the results.
    // TODO Gitignore benefits from processing files in particular order, which I (now) completely ignore. Some optimisation will be necessary.
    fn fuzzy_file_paths_it(&self, query: String, limit: usize, respect_ignores: bool) -> (LoadingState, Box<dyn Iterator<Item=Arc<PathBuf>> + '_>);

    fn is_ignored(&self, path: &Path) -> bool;

    //TODO:
    // - backup mechanism (don't loose data on crash)
    // - streaming save
    // - async save
    fn todo_save_file_sync(&self, path: &Path, bytes: &dyn AsRef<[u8]>) -> Result<(), std::io::Error>;

    fn overwrite_file(&self, path: &Path, source: &dyn SomethingToSave) -> Result<(), std::io::Error>;
}

