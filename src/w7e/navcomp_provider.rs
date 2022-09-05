use std::fmt::Debug;
use std::future::Future;

use async_trait::async_trait;

use crate::fs::path::SPath;
use crate::lsp_client::helpers::LspTextCursor;
use crate::primitives::cursor_set::Cursor;
use crate::w7e::navcomp_group::NavCompTickSender;

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

// this is a wrapper around LSP and "similar services".
#[async_trait]
pub trait NavCompProvider: Debug + Send + Sync {
    /*
    file_contents are strictly LSP requirement
     */
    fn file_open_for_edition(&self, path: &SPath, file_contents: String);

    /*
    I will add "incremental updates" at later stage.
     */
    fn submit_edit_event(&self, path: &SPath, file_contents: String);

    async fn completions(&self, path: SPath, cursor: LspTextCursor) -> Vec<Completion>;

    // TODO this will probably get more complicated
    fn completion_triggers(&self, path: &SPath) -> Vec<String>;

    fn file_closed(&self, path: &SPath);

    // fn todo_navcomp_sender(&self) -> &NavCompTickSender;
}