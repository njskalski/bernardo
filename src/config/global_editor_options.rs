use std::path::PathBuf;

use log::{debug, warn};
use regex::Regex;
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

            if let Ok(path) = which::which("clangd") {
                debug!("got it at [{:?}]", &path);
                return Some(path);
            }

            let re = Regex::new(r"/usr/lib/llvm-\d/bin/clangd$").unwrap();
            let mut binaries: Vec<PathBuf> = which::which_re(re).unwrap().collect();

            if binaries.is_empty() {
                debug!("couldn't find clangd neither on path nor in /usr/lib/llvm-*/bin/clangd");
                return None;
            }

            binaries.sort();

            debug!("found {} binaries at /usr/lib/llvm-*/bin/clangd, picking highest number", binaries.len());

            binaries.last().map(|item| item.clone())
        })
    }
}
