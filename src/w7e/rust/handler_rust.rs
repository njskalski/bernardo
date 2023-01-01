use std::sync::Arc;
use std::time::Duration;

use log::{debug, error};

use crate::config::config::ConfigRef;
use crate::fs::path::SPath;
use crate::gladius::sidechannel::x::SideChannel;
use crate::tsw::lang_id::LangId;
use crate::w7e::handler::{Handler, NavCompRef};
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_group::NavCompTickSender;
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
                sidechannel: SideChannel,
    ) -> Result<RustHandler, HandlerLoadError> {
        if !ff.is_dir() {
            return Err(HandlerLoadError::NotAProject);
        }

        let workspace_root = ff.absolute_path();

        let cargo_file = ff
            .descendant_checked("Cargo.toml")
            .ok_or(HandlerLoadError::NotAProject)?;
        if !cargo_file.is_file() {
            return Err(HandlerLoadError::NotAProject);
        }

        let contents = cargo_file.read_entire_file()?;
        let cargo = cargo_toml::Manifest::from_slice(&contents)
            .map_err(|e| HandlerLoadError::DeserializationError(e.to_string()))?;

        let mut navcomp_op: Option<NavCompRef> = None;

        #[cfg(not(test))]
        {
            let lsp_path = config.global.get_rust_lsp_path().ok_or(HandlerLoadError::LspNotFound)?;
            if let Some(navcomp_lsp) = NavCompProviderLsp::new(lsp_path, workspace_root, tick_sender) {
                navcomp_op = Some(Arc::new(Box::new(navcomp_lsp)));
            } else {
                error!("LspWrapper construction failed.")
            }
        }

        #[cfg(test)]
        {
            debug!("initializing MockNavCompProvider");
            let args = sidechannel.get_navcomp_prov_args();

            navcomp_op = Some(
                Arc::new(
                    Box::new(
                        crate::mocks::mock_navcomp_provider::MockNavCompProvider::new(
                            tick_sender.clone(),
                            args.0,
                            args.1,
                        )
                    ) as Box<dyn NavCompProvider>)
            )
        }

        Ok(RustHandler {
            root: ff,
            cargo_file: cargo,
            navcomp: navcomp_op,
        })
    }
}
