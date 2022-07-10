use crate::fs::file_front::FileFront;
use crate::lsp_client::lsp_client::LspWrapper;
use crate::w7e::handler::{Handler, NavCompRef};
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::handler_load_error::HandlerLoadError::ReadError;
use crate::w7e::navcomp_provider::NavCompProvider;
use crate::w7e::navcomp_provider_lsp::NavCompProviderLsp;
use crate::LangId;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct RustHandler {
    root: FileFront,
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
    pub fn load(ff: FileFront) -> Result<RustHandler, HandlerLoadError> {
        if !ff.is_dir() {
            return Err(HandlerLoadError::NotAProject);
        }

        let cargo_file = ff
            .descendant("Cargo.toml")
            .ok_or(HandlerLoadError::NotAProject)?;
        if !cargo_file.is_file() {
            return Err(HandlerLoadError::NotAProject);
        }

        let contents = cargo_file.read_entire_file_to_bytes()?;
        let cargo = cargo_toml::Manifest::from_slice(&contents)
            .map_err(|e| HandlerLoadError::DeserializationError(Box::new(e)))?;

        let lsp = LspWrapper::todo_new(ff.path_rc().to_path_buf()).map(|lsp| Arc::new(lsp));
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
