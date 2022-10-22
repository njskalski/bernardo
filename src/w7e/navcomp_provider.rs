use std::fmt::Debug;

use crate::fs::path::SPath;
use crate::lsp_client::helpers::LspTextCursor;
use crate::promise::promise::Promise;
use crate::w7e::navcomp_group::NavCompTickSender;

#[derive(Debug)]
pub enum NavcompError {
    UnmappedError(String)
}

pub type NavCompRes<T> = Result<NavcompError, T>;

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
pub enum Symbol {}

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

    fn todo_get_symbol_at(&self, path: &SPath, cursor: LspTextCursor) -> Option<Symbol>;

    fn file_closed(&self, path: &SPath);

    fn todo_navcomp_sender(&self) -> &NavCompTickSender;
}