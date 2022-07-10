use std::sync::Arc;

use crate::lsp_client::lsp_client::LspWrapper;
use crate::w7e::navcomp_provider::NavCompProvider;

pub struct NavCompProviderLsp {
    lsp: Arc<LspWrapper>,
}

impl NavCompProviderLsp {
    pub fn new(lsp: Arc<LspWrapper>) -> Self {
        NavCompProviderLsp { lsp }
    }
}

impl NavCompProvider for NavCompProviderLsp {}
