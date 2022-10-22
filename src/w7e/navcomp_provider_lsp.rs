use std::fmt::{Debug, Formatter};
use std::sync::{RwLock, RwLockWriteGuard};

use log::{debug, error};
use lsp_types::{CompletionResponse, CompletionTextEdit};

use crate::fs::path::SPath;
use crate::lsp_client::helpers::LspTextCursor;
use crate::lsp_client::lsp_client::LspWrapper;
use crate::promise::promise::Promise;
use crate::w7e::navcomp_group::NavCompTickSender;
use crate::w7e::navcomp_provider::{Completion, CompletionAction, CompletionsPromise, NavCompProvider, Symbol, SymbolContextActionsPromise};

/*
TODO I am silently ignoring errors here. I guess that if NavComp fails it should get re-started.
TODO (same as above) Use NavCompRes everywhere.
 */

pub struct NavCompProviderLsp {
    // TODO probably a RefCell would suffice
    lsp: RwLock<LspWrapper>,
    todo_tick_sender: NavCompTickSender,
    triggers: Vec<String>,
}

impl NavCompProviderLsp {
    pub fn new(lsp: LspWrapper, tick_sender: NavCompTickSender) -> Self {
        NavCompProviderLsp {
            lsp: RwLock::new(lsp),
            todo_tick_sender: tick_sender,
            // TODO this will get lang specific
            triggers: vec![".".to_string(), "::".to_string()],
        }
    }

    pub fn get_url_and_lock(&self, path: &SPath) -> Option<(url::Url, RwLockWriteGuard<LspWrapper>)> {
        let url = match path.to_url() {
            Ok(url) => url,
            Err(e) => {
                error!("failed to convert spath [{}] to url", path);
                return None;
            }
        };

        match self.lsp.try_write() {
            Err(e) => {
                error!("failed acquiring write lock: {}", e);
                None
            }
            Ok(lock) => {
                Some((url, lock))
            }
        }
    }
}


impl NavCompProvider for NavCompProviderLsp {
    fn file_open_for_edition(&self, path: &SPath, file_contents: String) {
        self.get_url_and_lock(path).map(|(url, mut lock)| {
            lock.text_document_did_open(url, file_contents);
        });
    }

    fn submit_edit_event(&self, path: &SPath, file_contents: String) {
        self.get_url_and_lock(path).map(|(url, mut lock)| {
            lock.text_document_did_change(url, file_contents);
        });
    }

    fn completions(&self, path: SPath, cursor: LspTextCursor, _trigger: Option<String>) -> Option<CompletionsPromise> {
        self.get_url_and_lock(&path).map(|(url, mut lock)| {
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
                })) as Box<dyn Promise<Vec<Completion>> + 'static>),
            }
        }).flatten()
    }

    fn completion_triggers(&self, _path: &SPath) -> &Vec<String> {
        &self.triggers
    }

    fn todo_get_context_options(&self, path: &SPath, cursor: LspTextCursor) -> Option<SymbolContextActionsPromise> {
        todo!()
        // let url = match path.to_url() {
        //     Ok(url) => url,
        //     Err(_) => {
        //         error!("failed opening for edition, because path->url cast failed.");
        //         return None;
        //     }
        // };
        //
        // match self.lsp.try_write() {
        //     Err(_) => {
        //         // this should never happen
        //         error!("failed acquiring write lock");
        //         None
        //     }
        //     Ok(mut lock) => {
        //         match lock.text_document_completion(url, cursor, true /*TODO*/, None /*TODO*/) {
        //             Err(e) => {
        //                 error!("failed sending text_document_completion: {:?}", e);
        //                 None
        //             }
        //             Ok(resp) => Some(Box::new(resp.map(|cop| -> Vec<Completion> {
        //                 match cop {
        //                     None => vec![],
        //                     Some(comps) => {
        //                         match comps {
        //                             CompletionResponse::Array(arr) => {
        //                                 arr.into_iter().map(translate_completion_item).collect()
        //                             }
        //                             CompletionResponse::List(list) => {
        //                                 // TODO is complete ignored
        //                                 list.items.into_iter().map(translate_completion_item).collect()
        //                             }
        //                         }
        //                     }
        //                 }
        //             }))),
        //         }
        //     }
        // }
    }

    fn todo_get_symbol_at(&self, path: &SPath, cursor: LspTextCursor) -> Option<Symbol> {
        todo!()
    }

    fn file_closed(&self, _path: &SPath) {
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

// TODO convert suggestions to CEMs?

fn translate_completion_item(i: lsp_types::CompletionItem) -> Completion {
    debug!("[{:?}]", i);
    Completion {
        key: i.label,
        desc: i.detail,
        action: CompletionAction::Insert(i.text_edit.map(|c|
            match c {
                CompletionTextEdit::Edit(e) => {
                    e.new_text
                }
                CompletionTextEdit::InsertAndReplace(e) => {
                    e.new_text
                }
            }
        ).unwrap_or("".to_string())),
    }
}