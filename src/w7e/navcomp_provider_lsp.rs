use std::sync::Arc;

use crate::lsp_client::lsp_client::{LspWrapper, LspWrapperRef};
use crate::w7e::navcomp_provider::NavCompProvider;

pub struct NavCompProviderLsp {
    lsp: LspWrapperRef,
}

impl NavCompProviderLsp {
    pub fn new(lsp: LspWrapperRef) -> Self {
        NavCompProviderLsp { lsp }
    }
}

impl NavCompProvider for NavCompProviderLsp {}
