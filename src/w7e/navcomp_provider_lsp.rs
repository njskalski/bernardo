use std::fmt::{Debug, Formatter};
use std::sync::{RwLock, TryLockResult};
use std::thread;

use log::{debug, error, warn};
use lsp_types::CompletionResponse;

use crate::fs::path::SPath;
use crate::lsp_client::helpers::LspTextCursor;
use crate::lsp_client::lsp_client::LspWrapper;
use crate::lsp_client::lsp_io_error::LspIOError;
use crate::lsp_client::lsp_write_error::LspWriteError;
use crate::lsp_client::promise::LSPPromise;
use crate::primitives::cursor_set::Cursor;
use crate::promise::promise::Promise;
use crate::w7e::navcomp_group::NavCompTickSender;
use crate::w7e::navcomp_provider::{Completion, CompletionAction, CompletionsPromise, NavCompProvider};

/*
This is work in progress. I do not know mutability rules. I am also not sure if I don't want
to introduce some channels to facilitate async communication. Right now I just call async functions
from non-async and hope it works out.
 */
pub struct NavCompProviderLsp {
    // TODO probably a RefCell would suffice
    lsp: RwLock<LspWrapper>,
    todo_tick_sender: NavCompTickSender,
}

impl NavCompProviderLsp {
    pub fn new(lsp: LspWrapper, tick_sender: NavCompTickSender) -> Self {
        NavCompProviderLsp {
            lsp: RwLock::new(lsp),
            todo_tick_sender: tick_sender,
        }
    }
}

impl NavCompProvider for NavCompProviderLsp {
    fn file_open_for_edition(&self, path: &SPath, file_contents: String) {
        let url = match path.to_url() {
            Ok(url) => url,
            Err(_) => {
                error!("failed opening for edition, because path->url cast failed.");
                return;
            }
        };

        match self.lsp.try_write() {
            Err(_) => {
                // this should never happen
                error!("failed acquiring write lock");
            }
            Ok(mut lock) => {
                match lock.text_document_did_open(url.clone(), file_contents) {
                    Ok(_) => {}
                    Err(_) => error!("failed sending text_document_did_open"),
                }
            }
        }
    }

    fn submit_edit_event(&self, path: &SPath, file_contents: String) {
        let url = match path.to_url() {
            Ok(url) => url,
            Err(_) => {
                error!("failed opening for edition, because path->url cast failed.");
                return;
            }
        };

        match self.lsp.try_write() {
            Err(_) => {
                // this should never happen
                error!("failed acquiring write lock");
            }
            Ok(mut lock) => {
                match lock.text_document_did_change(url.clone(), file_contents) {
                    Ok(_) => {}
                    Err(_) => error!("failed sending text_document_did_change"),
                }
            }
        }
    }

    fn completions(&self, path: SPath, cursor: LspTextCursor) -> Option<CompletionsPromise> {
        let url = match path.to_url() {
            Ok(url) => url,
            Err(_) => {
                error!("failed opening for edition, because path->url cast failed.");
                return None;
            }
        };

        match self.lsp.try_write() {
            Err(_) => {
                // this should never happen
                error!("failed acquiring write lock");
                None
            }
            Ok(mut lock) => {
                match lock.text_document_completion(url, cursor, true /*TODO*/, None /*TODO*/) {
                    Err(e) => {
                        error!("failed sending text_document_completion: {:?}", e);
                        None
                    }
                    Ok(resp) => Some(Box::new(resp.map(|cop| -> Vec<Completion> {
                        match cop {
                            None => vec![],
                            Some(comps) => {
                                match comps {
                                    CompletionResponse::Array(arr) => {
                                        arr.into_iter().map(translate_completion_item).collect()
                                    }
                                    CompletionResponse::List(list) => {
                                        // TODO is complete ignored
                                        list.items.into_iter().map(translate_completion_item).collect()
                                    }
                                }
                            }
                        }
                    }))),
                }
            }
        }
    }

    fn completion_triggers(&self, _path: &SPath) -> Vec<String> {
        //TODO this will get language specific
        vec![".".to_string(), "::".to_string()]
    }

    fn file_closed(&self, path: &SPath) {
        todo!()
    }

    fn todo_navcomp_sender(&self) -> &NavCompTickSender {
        &self.todo_tick_sender
    }
}

impl Debug for NavCompProviderLsp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NavComp({:?})", self.lsp)
    }
}

fn translate_completion_item(i: lsp_types::CompletionItem) -> Completion {
    Completion {
        key: i.label,
        desc: i.detail,
        action: CompletionAction::Insert(i.insert_text.unwrap_or("".to_string())),
    }
}