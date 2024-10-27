use crate::cursor::cursor::Cursor;
use crate::experiments::focus_group::FocusUpdate;
use crate::fs::path::SPath;
use crate::widget::any_msg::AnyMsg;
use crate::widgets::code_results_view::stupid_symbol_usage_code_results_provider::StupidSymbolUsageCodeResultsProvider;
use crate::widgets::main_view::main_view::DocumentIdentifier;

#[derive(Debug)]
pub enum MainViewMsg {
    FocusUpdateMsg(FocusUpdate),

    // This is called whenever item in tree is "expanded" or "collapsed".
    // I'm moving entire ChildRc, because PathBuf would allocate, and passing &Path would unleash borrow checker hell.
    TreeExpandedFlip {
        expanded: bool,
        item: SPath,
    },

    // This is called whenever a file is selected.
    TreeSelected {
        item: SPath,
    },

    OpenNewFile,

    // Open fuzzy files
    OpenFuzzyFiles,
    // Used by OpenOpenBuffers too
    CloseHover,

    // Open "open buffers"
    OpenFuzzyBuffers,
    FuzzyBuffersHit {
        pos: usize,
    },

    // it's option, just that we can "take" it, not changing the msg, because that doesn't work well
    FindReferences {
        promise_op: Option<StupidSymbolUsageCodeResultsProvider>,
    },

    /*
    file, or identifier of scratchpad (to be filled)
     */
    OpenFile {
        file: DocumentIdentifier,
        position_op: Option<Cursor>,
    },

    BufferChangedName {
        updated_identifier: DocumentIdentifier,
    },

    OpenFindInFiles {
        root_dir: SPath,
    },
    FindInFilesQuery {
        root_dir: SPath,
        query: String,
        filter_op: Option<String>,
    },

    GoToDefinition {
        promise_op: Option<StupidSymbolUsageCodeResultsProvider>,
    },
}

impl AnyMsg for MainViewMsg {}
