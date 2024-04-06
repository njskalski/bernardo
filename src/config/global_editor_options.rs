use std::path::PathBuf;

use log::{debug, warn};
use serde::{Deserialize, Serialize};
use which;

use crate::unpack_or_e;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct GlobalEditorOptions {
    pub rust_lsp_path: Option<PathBuf>,
    pub clangd_lsp_path: Option<PathBuf>,
}

impl GlobalEditorOptions {
    pub fn get_rust_lsp_path(&self) -> Option<PathBuf> {
        self.rust_lsp_path.clone().or_else(|| {
            debug!("discovering location of rust-analyzer");

            let mut paths_iter = unpack_or_e!(which::which_all("rust-analyzer").ok(), None, "failed to query for rust-analyzer");
            let mut paths: Vec<PathBuf> = paths_iter.collect();

            if paths.len() > 1 {
                warn!(
                    "multiple paths to rust_analyzer found, will default to first one [{:?}] from [{:?}]",
                    paths.first().unwrap(),
                    paths
                );
            }

            let first_path = unpack_or_e!(paths.into_iter().next(), None, "failed to find any instance of rust-analyzer");

            warn!("using rust-analyzer from {:?}", &first_path);

            Some(first_path)
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
