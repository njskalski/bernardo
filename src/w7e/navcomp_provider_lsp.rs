use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use log::{debug, error};
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
    fn file_open_for_edition(&self, path: &SPath) {
        let url = match path.to_url() {
            Ok(url) => url,
            Err(_) => {
                error!("failed opening for edition, because path->url cast failed.");
                return;
            }
        };

        self.lsp.try_borrow_mut().map(|mut lsp| {
            tokio::spawn(|| lsp.text_document_did_open(url));
        }).unwrap_or_else(|| {
            debug!("failed sending file_open_for_edition - can't acquire LSP ref");
        })
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