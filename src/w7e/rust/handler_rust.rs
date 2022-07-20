use std::path::PathBuf;
use std::sync::Arc;

use crate::fs::file_front::FileFront;
use crate::LangId;
use crate::lsp_client::lsp_client::LspWrapper;
use crate::new_fs::path::SPath;
use crate::w7e::handler::{Handler, NavCompRef};
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_provider::NavCompProvider;
use crate::w7e::navcomp_provider_lsp::NavCompProviderLsp;

pub struct RustHandler {
    root: SPath,
    cargo_file: cargo_toml::Manifest,

    // TODO merge these two?
    lsp: Option<Arc<LspWrapper>>,
    navcomp: Option<NavCompRef>,
}

impl Handler for RustHandler {
    fn lang_id(&self) -> LangId {
        LangId::RUST
    }

    fn handler_id(&self) -> &'static str {
        "rust"
    }

    fn project_name(&self) -> &str {
        "todo"
    }

    fn navcomp(&self) -> Option<NavCompRef> {
        self.navcomp.clone()
    }
}

impl RustHandler {
    pub fn load(ff: SPath) -> Result<RustHandler, HandlerLoadError> {
        if !ff.is_dir() {
            return Err(HandlerLoadError::NotAProject);
        }

        let cargo_file = ff
            .descendant_checked("Cargo.toml")
            .ok_or(HandlerLoadError::NotAProject)?;
        if !cargo_file.is_file() {
            return Err(HandlerLoadError::NotAProject);
        }

        let contents = cargo_file.read_entire_file()?;
        let cargo = cargo_toml::Manifest::from_slice(&contents)
            .map_err(|e| HandlerLoadError::DeserializationError(Box::new(e)))?;

        let lsp = LspWrapper::todo_new(ff.relative_path()).map(|lsp| Arc::new(lsp));
        let navcomp = lsp.clone().map(|lsp| {
            Arc::new(Box::new(NavCompProviderLsp::new(lsp)) as Box<dyn NavCompProvider>)
        });

        Ok(RustHandler {
            root: ff,
            cargo_file: cargo,
            lsp,
            navcomp,
        })
    }
}
