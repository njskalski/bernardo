use std::sync::Arc;
use crate::lsp_client::lsp_client::LspWrapper;

pub struct NavCompProviderLsp {
    lsp: Arc<LspWrapper>,
}