use std::fmt::{Debug, Formatter};
use std::future::Future;

use async_trait::async_trait;
use log::{debug, error, warn};
use lsp_types::CompletionResponse;
use tokio::spawn;

use crate::fs::path::SPath;
use crate::lsp_client::helpers::LspTextCursor;
use crate::lsp_client::lsp_client::LspWrapperRef;
use crate::lsp_client::lsp_io_error::LspIOError;
use crate::primitives::cursor_set::Cursor;
use crate::w7e::navcomp_provider::{Completion, CompletionAction, NavCompProvider};

/*
This is work in progress. I do not know mutability rules. I am also not sure if I don't want
to introduce some channels to facilitate async communication. Right now I just call async functions
from non-async and hope it works out.
 */
pub struct NavCompProviderLsp {
    lsp: LspWrapperRef,
}

impl NavCompProviderLsp {
    pub fn new(lsp: LspWrapperRef) -> Self {
        NavCompProviderLsp { lsp }
    }
}

#[async_trait(? Send)]
impl NavCompProvider for NavCompProviderLsp {
    fn file_open_for_edition(&self, path: &SPath, file_contents: String) {
        let url = match path.to_url() {
            Ok(url) => url,
            Err(_) => {
                error!("failed opening for edition, because path->url cast failed.");
                return;
            }
        };

        let lsp = self.lsp.clone();

        let item = spawn(async move {
            let mut lsp_lock = lsp.write().await;
            lsp_lock.text_document_did_open(url.clone(), file_contents).await.map_err(|err| {
                debug!("failed sending text_document_did_open for {}", url);
            })
        });
    }

    fn submit_edit_event(&self, path: &SPath, file_contents: String) {
        let url = match path.to_url() {
            Ok(url) => url,
            Err(_) => {
                error!("failed opening for edition, because path->url cast failed.");
                return;
            }
        };

        let lsp = self.lsp.clone();

        let item = spawn(async move {
            let mut lsp_lock = lsp.write().await;
            lsp_lock.text_document_did_change(url.clone(), file_contents).await.map_err(|err| {
                debug!("failed sending text_document_did_change for {}", url);
            })
        });
    }

    async fn completions(&self, path: &SPath, cursor: LspTextCursor) -> Vec<Completion> {
        let url = match path.to_url() {
            Ok(url) => url,
            Err(_) => {
                error!("failed opening for edition, because path->url cast failed.");
                return vec![];
            }
        };

        let lsp_arc = self.lsp.clone();
        let mut lsp = lsp_arc.write().await;
        match lsp.text_document_completion(url, cursor, true /*TODO*/, None).await {
            Ok(resp) => {
                match resp {
                    None => {
                        warn!("no response for completion request");
                        Vec::new()
                    }
                    Some(response) => {
                        match response {
                            CompletionResponse::Array(arr) => {
                                arr.into_iter().map(translate_completion_item).collect()
                            }
                            CompletionResponse::List(list) => {
                                list.items.into_iter().map(translate_completion_item).collect()
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("failed retrieving completions: {:?}", e);
                Vec::new()
            }
        }
    }

    fn file_closed(&self, path: &SPath) {
        todo!()
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