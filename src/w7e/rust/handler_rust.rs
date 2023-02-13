use std::time::Duration;

use crate::config::config::ConfigRef;
use crate::fs::path::SPath;
use crate::tsw::lang_id::LangId;
use crate::w7e::handler::{Handler, NavCompRef};
use crate::w7e::handler_load_error::HandlerLoadError;

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
 */
impl RustHandler {
    pub fn load(_config: &ConfigRef,
                ff: SPath,
                navcomp_op: Option<NavCompRef>,
    ) -> Result<RustHandler, HandlerLoadError> {
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
            .map_err(|e| HandlerLoadError::DeserializationError(e.to_string()))?;

        Ok(RustHandler {
            root: ff,
            cargo_file: cargo,
            navcomp: navcomp_op,
        })
    }
}
