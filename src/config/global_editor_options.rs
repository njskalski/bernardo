use std::path::PathBuf;

use log::{debug, warn};
use serde::{Deserialize, Serialize};
use which;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct GlobalEditorOptions {
    pub rust_lsp_path: Option<PathBuf>,
    pub clangd_lsp_path: Option<PathBuf>,
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
                    warn!("did not find [rust-analyzer], because: {}", e);
                    None
                }
            }
        })
    }

    pub fn get_clangd_lsp_path(&self) -> Option<PathBuf> {
        self.rust_lsp_path.as_ref().map(|c| c.clone()).or_else(|| {
            debug!("discovering location of clangd");

            match which::which("clangd") {
                Ok(item) => {
                    debug!("got it at [{:?}]", &item);
                    Some(item)
                }
                Err(e) => {
                    warn!("did not find [clangd], because: {}", e);
                    None
                }
            }
        })
    }
}
