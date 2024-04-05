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
        self.rust_lsp_path.clone().or_else(|| {
            debug!("discovering location of rust_analyzer");

            match which::which("rust-analyzer") {
                Ok(item) => {
                    debug!("got it at [{:?}]", &item);
                    Some(item)
                }
                Err(e) => {
                    warn!("did not find [rust-analyzer], because: {}", e);

                    // TODO this is a quick and dirty hack because "which" fails
                    if std::fs::metadata("/home/andrzej/.cargo/bin/rust-analyzer").is_ok() {
                        Some(PathBuf::from("/home/andrzej/.cargo/bin/rust-analyzer"))
                    } else {
                        None
                    }
                }
            }
        })
    }

    pub fn get_clangd_lsp_path(&self) -> Option<PathBuf> {
        self.rust_lsp_path.clone().or_else(|| {
            debug!("discovering location of clangd");

            if let Ok(path) = which::which("clangd") {
                debug!("got it at [{:?}]", &path);
                return Some(path);
            }

            for idx in (10..20).rev() {
                let path: PathBuf = PathBuf::from(format!("/usr/lib/llvm-{}/bin/clangd", idx));
                debug!("checking at {}", path.to_str().unwrap());
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.is_file() {
                        debug!("found clang at {}", path.to_str().unwrap_or("[failed to unwrap as str]"));
                        return Some(path);
                    }
                }
            }

            debug!("couldn't find clangd neither on path nor in /usr/lib/llvm-*/bin/clangd");
            None
        })
    }
}
