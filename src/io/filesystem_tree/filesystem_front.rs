use std::cell::RefCell;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ropey::Rope;

use crate::io::filesystem_tree::file_front::FileFront;
use crate::text::buffer_state::BufferState;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

pub type FsfRef = Rc<Box<dyn FilesystemFront>>;

pub trait FilesystemFront {
    fn get_root(&self) -> Rc<FileFront>;

    // This is a mock method. It should probably return a stream and should probably report errors.
    // One of many "nice to haves" of this editor, outside of scope of MVP, is "large files support",
    // that I want to test with infinite file generator behind an interface here.
    fn todo_read_file(&self, path: &Path) -> Result<Rope, ()>;

    fn is_dir(&self, path: &Path) -> bool;

    // first argument says if the list is complete.
    // none = true, empty iterator
    fn get_children(&self, path: &Path) -> (bool, Box<dyn Iterator<Item=Rc<FileFront>>>);
}

