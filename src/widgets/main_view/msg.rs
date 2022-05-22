use crate::AnyMsg;
use crate::experiments::focus_group::FocusUpdate;
use crate::fs::file_front::FileFront;

#[derive(Clone, Debug)]
pub enum MainViewMsg {
    FocusUpdateMsg(FocusUpdate),

    // This is called whenever item in tree is "expanded" or "collapsed".
    // I'm moving entire ChildRc, because PathBuf would allocate, and passing &Path would unleash borrow checker hell.
    TreeExpandedFlip { expanded: bool, item: FileFront },

    // This is called whenever a file is selected.
    TreeSelected { item: FileFront },

    OpenNewFile,

    // Open fuzzy files
    OpenFuzzyFiles,
    // Used by OpenOpenBuffers too
    ClozeHover,

    // Open "open buffers"
    OpenFuzzyBuffers,
    FuzzyBuffersHit { pos: usize }
}

impl AnyMsg for MainViewMsg {}
