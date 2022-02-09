use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ropey::Rope;

use crate::io::filesystem_tree::filesystem_list_item::FilesystemListItem;
use crate::text::buffer_state::BufferState;
use crate::widgets::tree_view::tree_view_node::TreeViewNode;

pub trait FilesystemProvider {
    fn get_root(&self) -> Rc<dyn TreeViewNode<PathBuf>>;

    fn expand(&mut self, path: &Path) -> bool;

    fn get_files(&self, path: &Path) -> Box<dyn Iterator<Item=FilesystemListItem>>;

    // This is a mock method. It should probably return a stream and should probably report errors.
    // One of many "nice to haves" of this editor, outside of scope of MVP, is "large files support",
    // that I want to test with infinite file generator behind an interface here.
    fn todo_read_file(&mut self, path: &Path) -> Result<Rope, ()>;
}

