use std::fmt::Debug;
use std::ops::Range;

use crate::fs::path::SPath;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::promise::promise::Promise;
use crate::w7e::navcomp_group::NavCompTickSender;

// So I am not sure if I want to escalate errors from underlying implementation (LSP most likely)
//  or just provide some generic "check health" status, that would trigger a reload when LSP dies.
// The latter seems cleaner than just escalating all results to be what, handled, ignored in UI?

// Yeah, I get this feeling more and more. It's a programmers' move to deliver results of both
//  successful and failed calls to the same place, which results in terrible UX.
// A successful navcomp call results in options available to user.
// Failed navcomp calls should lead to logs, automated restart of underlying service, and if problem
//  persists a tiny tiny warning to the user "hey, this shit is so broken I can't cope with it".
//
// Bottom line, if your UX designer comes up with a popup
//                                                        then you should fire your UX designer.
// Programmers are allowed to make such mistakes. Look on how we dress and ask yourself: "would
//  I take esthetics advice from this person?"

// Also, I am tired of making boilerplate code here between LSP and "other navcomps", that I am not
//  sure will ever exist. But then I recall that without it, I'd be exposed to whatever becomes of
//  LSP in the future.

#[derive(Debug, Clone)]
pub enum CompletionAction {
    Insert(String)
}

#[derive(Debug, Clone)]
pub struct Completion {
    pub key: String,
    pub desc: Option<String>,
    pub action: CompletionAction,
}

pub type CompletionsPromise = Box<dyn Promise<Vec<Completion>> + 'static>;

// Currently these map LSP types 1:1, but this might change. Most importantly I have a feeling I
//  might prefer to use tree-sitter symbols instead.
#[derive(Debug, Clone)]
pub enum SymbolType {
    File,
    Module,
    Namespace,
    Package,
    Class,
    Method,
    Property,
    Field,
    Constructor,
    Enum,
    Interface,
    Function,
    Variable,
    Constant,
    String,
    Number,
    Boolean,
    Array,
    Object,
    Key,
    Null,
    EnumMember,
    Event,
    Struct,
    Operator,
    TypeParameter,
    Unmapped(String),
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub symbol_type: SymbolType,
    pub stupid_range: (StupidCursor, StupidCursor),
}

/*
This is super work in progress, I added some top of the head options to "smoke out" what they imply.
 */
#[derive(Debug, Clone)]
pub enum NavCompSymbolContextActions {
    GoToDefinition,
    FindUsages,
    NextUsage,
    PrevUsage,
}

pub type SymbolContextActionsPromise = Box<dyn Promise<Vec<Completion>> + 'static>;
pub type SymbolPromise = Box<dyn Promise<Option<Symbol>> + 'static>;

// this is a wrapper around LSP and "similar services".
pub trait NavCompProvider: Debug {
    /*
    file_contents are strictly LSP requirement
     */
    fn file_open_for_edition(&self, path: &SPath, file_contents: ropey::Rope);

    /*
    I will add "incremental updates" at later stage.
     */
    fn submit_edit_event(&self, path: &SPath, file_contents: ropey::Rope);

    fn completions(&self, path: SPath, cursor: StupidCursor, trigger: Option<String>) -> Option<CompletionsPromise>;

    // TODO this will probably get more complicated
    fn completion_triggers(&self, path: &SPath) -> &Vec<String>;

    fn todo_get_context_options(&self, path: &SPath, cursor: StupidCursor) -> Option<SymbolContextActionsPromise>;

    fn todo_get_symbol_at(&self, path: &SPath, cursor: StupidCursor) -> Option<SymbolPromise>;

    fn file_closed(&self, path: &SPath);

    fn todo_navcomp_sender(&self) -> &NavCompTickSender;

    fn todo_is_healthy(&self) -> bool;
}