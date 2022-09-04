use std::fmt::Debug;

use async_trait::async_trait;

use crate::fs::path::SPath;
use crate::lsp_client::helpers::LspTextCursor;
use crate::primitives::cursor_set::Cursor;

pub enum CompletionAction {
    Insert(String)
}

pub struct Completion {
    pub key: String,
    pub desc: Option<String>,
    pub action: CompletionAction,
}

// this is a wrapper around LSP and "similar services".
#[async_trait(? Send)]
pub trait NavCompProvider: Debug {
    /*
    file_contents are strictly LSP requirement
     */
    fn file_open_for_edition(&self, path: &SPath, file_contents: String);

    /*
    I will add "incremental updates" at later stage.
     */
    fn submit_edit_event(&self, path: &SPath, file_contents: String);

    async fn completions(&self, path: &SPath, cursor: LspTextCursor) -> Vec<Completion>;

    fn file_closed(&self, path: &SPath);
}