use std::path::PathBuf;

use crate::AnyMsg;
use crate::experiments::focus_group::FocusUpdate;
use crate::widgets::tree_view::tree_view_node::ChildRc;

#[derive(Clone, Debug)]
pub enum MainViewMsg {
    FocusUpdateMsg(FocusUpdate),

    // This is called whenever item in tree is "expanded" or "collapsed".
    // I'm moving entire ChildRc, because PathBuf would allocate, and passing &Path would unleash borrow checker hell.
    TreeExpandedFlip { expanded: bool, item: ChildRc<PathBuf> },

    // This is called whenever a file is selected.
    TreeSelected { item: ChildRc<PathBuf> },
}

impl AnyMsg for MainViewMsg {}