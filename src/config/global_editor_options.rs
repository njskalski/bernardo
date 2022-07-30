use std::path::PathBuf;

use log::debug;
use serde::{Deserialize, Serialize};
use which;

use crate::primitives::is_default::IsDefault;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct GlobalEditorOptions {
    pub rust_lsp_path: Option<PathBuf>,
}

impl GlobalEditorOptions {
    pub fn get_rust_lsp_path(&self) -> Option<PathBuf> {
        self.rust_lsp_path.as_ref().map(|c| c.clone()).or_else(|| {
            debug!("discovering location of rust_analyzer");

            match which::which("rust-analyzer") {
                Ok(item) => {
                    debug!("got it at [{:?}]", &item);
                    Some(item)
                }
                Err(e) => {
                    debug!("did not find [rust-analyzer], because: {}", e);
                    None
                }
            }
        })
    }
}
