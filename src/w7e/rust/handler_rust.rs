use std::sync::Arc;
use std::time::Duration;

use log::{debug, error};

use crate::{ConfigRef, LangId};
use crate::fs::path::SPath;
use crate::lsp_client::lsp_client::LspWrapper;
use crate::w7e::handler::{Handler, NavCompRef};
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_group::{NavCompTick, NavCompTickSender};
use crate::w7e::navcomp_provider::NavCompProvider;
use crate::w7e::navcomp_provider_lsp::NavCompProviderLsp;

pub const INIT_TIMEOUT: Duration = Duration::from_millis(2000);

pub struct RustHandler {
    root: SPath,
    cargo_file: cargo_toml::Manifest,

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
    pub fn load(config: &ConfigRef,
                ff: SPath,
                tick_sender: NavCompTickSender,
    ) -> Result<RustHandler, HandlerLoadError> {
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

        let mut lsp_ref_op: Option<LspWrapper> = None;
        let mut navcomp_op: Option<NavCompRef> = None;

        if let Some(mut lsp) = LspWrapper::new(lsp_path,
                                               ff.absolute_path(),
                                               tick_sender.clone(),
        ) {
            // TODO reintroduce timeout?
            debug!("initializing lsp");
            match lsp.initialize() {
                Ok(_init_result) => {
                    debug!("lsp initialized successfully.");

                    navcomp_op = Some(
                        Arc::new(Box::new(NavCompProviderLsp::new(lsp, tick_sender.clone())) as Box<dyn NavCompProvider>)
                    );
                }
                Err(e) => {
                    error!("Lsp init failed: {:?}", e);
                }
            }
        } else {
            error!("LspWrapper construction failed.")
        }


        Ok(RustHandler {
            root: ff,
            cargo_file: cargo,
            navcomp: navcomp_op,
        })
    }
}
