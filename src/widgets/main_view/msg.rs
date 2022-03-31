use std::rc::Rc;

use crate::AnyMsg;
use crate::experiments::focus_group::FocusUpdate;
use crate::io::filesystem_tree::file_front::FileFront;

#[derive(Clone, Debug)]
pub enum MainViewMsg {
    FocusUpdateMsg(FocusUpdate),

    // This is called whenever item in tree is "expanded" or "collapsed".
    // I'm moving entire ChildRc, because PathBuf would allocate, and passing &Path would unleash borrow checker hell.
    TreeExpandedFlip { expanded: bool, item: FileFront },

    // This is called whenever a file is selected.
    TreeSelected { item: FileFront },
}

impl AnyMsg for MainViewMsg {}