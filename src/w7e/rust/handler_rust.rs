use std::cell::RefCell;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, error};
use lsp_types::InitializeResult;
use tokio::sync::RwLock;
use tokio::time::error::Elapsed;

use crate::{ConfigRef, LangId};
use crate::fs::path::SPath;
use crate::lsp_client::lsp_client::{LspWrapper, LspWrapperRef};
use crate::lsp_client::lsp_io_error::LspIOError;
use crate::w7e::handler::{Handler, NavCompRef};
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_provider::NavCompProvider;
use crate::w7e::navcomp_provider_lsp::NavCompProviderLsp;

pub const INIT_TIMEOUT: Duration = Duration::from_millis(400);

pub struct RustHandler {
    root: SPath,
    cargo_file: cargo_toml::Manifest,

    // TODO merge these two?
    lsp: Option<LspWrapperRef>,
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

/*
So handler can "partially work", meaning for instance that running/debugging works, but LSP does not.

 */
impl RustHandler {
    pub async fn load(config: &ConfigRef, ff: SPath) -> Result<RustHandler, HandlerLoadError> {
        if !ff.is_dir() {
            return Err(HandlerLoadError::NotAProject);
        }
        let lsp_path = config.global.get_rust_lsp_path().ok_or(HandlerLoadError::LspNotFound)?;

        let cargo_file = ff
            .descendant_checked("Cargo.toml")
            .ok_or(HandlerLoadError::NotAProject)?;
        if !cargo_file.is_file() {
            return Err(HandlerLoadError::NotAProject);
        }

        let contents = cargo_file.read_entire_file()?;
        let cargo = cargo_toml::Manifest::from_slice(&contents)
            .map_err(|e| HandlerLoadError::DeserializationError(e.to_string()))?;

        let mut lsp_ref_op: Option<LspWrapperRef> = None;
        let mut navcomp_op: Option<NavCompRef> = None;

        if let Some(mut lsp) = LspWrapper::new(lsp_path, ff.absolute_path()) {
            debug!("initializing lsp");
            if let Ok(res) = tokio::time::timeout(INIT_TIMEOUT, lsp.initialize()).await {
                match res {
                    Ok(init_result) => {
                        debug!("lsp initialized successfully.");

                        let arc_lsp = Arc::new(RwLock::new(lsp));
                        lsp_ref_op = Some(arc_lsp.clone());
                        navcomp_op = Some(
                            Arc::new(Box::new(NavCompProviderLsp::new(arc_lsp)) as Box<dyn NavCompProvider>)
                        );
                    }
                    Err(e) => {
                        error!("Lsp init failed: {:?}", e);
                    }
                }
            } else {
                error!("timed out waiting for LSP.")
            }
        } else {
            error!("LspWrapper construction failed.")
        }


        Ok(RustHandler {
            root: ff,
            cargo_file: cargo,
            lsp: lsp_ref_op,
            navcomp: navcomp_op,
        })
    }
}
