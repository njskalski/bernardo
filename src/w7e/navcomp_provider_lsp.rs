use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use log::{debug, error};
use tokio::spawn;
use tokio::task::spawn_blocking;
use url::Url;

use crate::fs::path::SPath;
use crate::lsp_client::lsp_client::{LspWrapper, LspWrapperRef};
use crate::w7e::navcomp_provider::NavCompProvider;

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

    fn file_closed(&self, path: &SPath) {
        todo!()
    }
}

impl Debug for NavCompProviderLsp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NavComp({:?})", self.lsp)
    }
}