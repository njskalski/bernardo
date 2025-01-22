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
    OpenContextMenu,
    // depth describes depth of focus path. It's Option so I can "take"
    ContextMenuHit {
        msg: Option<Box<dyn AnyMsg>>,
        depth: usize,
    },

    // Used by OpenOpenBuffers too
    CloseHover,
    CloseBuffer,

    // Open "open buffers"
    OpenChooseDisplay,
    ChooseDisplayHit {
        display_idx: usize,
    },
    NextDisplay,
    PrevDisplay,

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

    /*
    Opens file by path. Reopens existing buffer if it already exists.
     */
    OpenFileBySpath {
        spath: SPath,
    },

    BufferChangedName {
        updated_identifier: DocumentIdentifier,
    },

    OpenFindInFiles,
    FindInFilesQuery {
        root_dir: SPath,
        query: String,
        filter_op: Option<String>,
    },

    GoToDefinition {
        promise_op: Option<StupidSymbolUsageCodeResultsProvider>,
    },

    PruneUnchangedBuffers,

    QuitGladius,
}

impl AnyMsg for MainViewMsg {}
