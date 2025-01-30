use std::time::Duration;

use crate::config::config::ConfigRef;
use crate::fs::path::SPath;
use crate::tsw::lang_id::LangId;
use crate::w7e::handler::{Handler, NavCompRef};
use crate::w7e::handler_load_error::HandlerLoadError;

pub const INIT_TIMEOUT: Duration = Duration::from_millis(2000);

pub struct GolangHandler {
    root: SPath,

    navcomp: Option<NavCompRef>,
}

impl Handler for GolangHandler {
    fn lang_id(&self) -> LangId {
        LangId::GO
    }

    fn handler_id(&self) -> &'static str {
        "go"
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
impl GolangHandler {
    pub fn load(_config: &ConfigRef, ff: SPath, navcomp_op: Option<NavCompRef>) -> Result<GolangHandler, HandlerLoadError> {
        if !ff.is_dir() {
            return Err(HandlerLoadError::NotAProject);
        }

        let compile_commands_file = ff.descendant_checked("go.mod").ok_or(HandlerLoadError::NotAProject)?;
        if !compile_commands_file.is_file() {
            return Err(HandlerLoadError::NotAProject);
        }

        // let contents = compile_commands_file.read_entire_file()?;

        Ok(GolangHandler {
            root: ff,
            navcomp: navcomp_op,
        })
    }
}
