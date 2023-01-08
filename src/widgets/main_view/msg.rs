use crate::experiments::focus_group::FocusUpdate;
use crate::fs::path::SPath;
use crate::w7e::navcomp_provider::SymbolUsagesPromise;
use crate::widget::any_msg::AnyMsg;

#[derive(Debug)]
pub enum MainViewMsg {
    FocusUpdateMsg(FocusUpdate),

    // This is called whenever item in tree is "expanded" or "collapsed".
    // I'm moving entire ChildRc, because PathBuf would allocate, and passing &Path would unleash borrow checker hell.
    TreeExpandedFlip { expanded: bool, item: SPath },

    // This is called whenever a file is selected.
    TreeSelected { item: SPath },

    OpenNewFile,

    // Open fuzzy files
    OpenFuzzyFiles,
    // Used by OpenOpenBuffers too
    CloseHover,

    // Open "open buffers"
    OpenFuzzyBuffers,
    FuzzyBuffersHit { pos: usize },

    FindReferences { promise_op: Option<SymbolUsagesPromise> },
}

impl AnyMsg for MainViewMsg {}
