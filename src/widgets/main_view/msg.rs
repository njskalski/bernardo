use std::rc::Rc;

use crate::AnyMsg;
use crate::experiments::focus_group::FocusUpdate;
use crate::fs::file_front::FileFront;
use crate::widgets::fuzzy_search::fsf_provider::FileFrontMsg;

#[derive(Clone, Debug)]
pub enum MainViewMsg {
    FocusUpdateMsg(FocusUpdate),

    // This is called whenever item in tree is "expanded" or "collapsed".
    // I'm moving entire ChildRc, because PathBuf would allocate, and passing &Path would unleash borrow checker hell.
    TreeExpandedFlip { expanded: bool, item: FileFront },

    // This is called whenever a file is selected.
    TreeSelected { item: FileFront },

    // Open fuzzy files
    OpenFuzzyFiles,
    FuzzyClose,
}

impl AnyMsg for MainViewMsg {}
