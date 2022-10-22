use std::fmt::Debug;

use crate::fs::path::SPath;
use crate::lsp_client::helpers::LspTextCursor;
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

#[derive(Debug, Clone)]
pub enum SymbolType {}

#[derive(Debug, Clone)]
pub struct Symbol {}

/*
This is super work in progress, I added some top of the head options to "smoke out" what they imply.
 */
#[derive(Debug, Clone)]
pub enum NavCompSymbolContextActions {
    GoToDefiniton,
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
    fn file_open_for_edition(&self, path: &SPath, file_contents: String);

    /*
    I will add "incremental updates" at later stage.
     */
    fn submit_edit_event(&self, path: &SPath, file_contents: String);

    fn completions(&self, path: SPath, cursor: LspTextCursor, trigger: Option<String>) -> Option<CompletionsPromise>;

    // TODO this will probably get more complicated
    fn completion_triggers(&self, path: &SPath) -> &Vec<String>;

    fn todo_get_context_options(&self, path: &SPath, cursor: LspTextCursor) -> Option<SymbolContextActionsPromise>;

    fn todo_get_symbol_at(&self, path: &SPath, cursor: LspTextCursor) -> Option<SymbolPromise>;

    fn file_closed(&self, path: &SPath);

    fn todo_navcomp_sender(&self) -> &NavCompTickSender;

    fn todo_is_healthy(&self) -> bool;
}