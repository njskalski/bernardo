use std::time::Duration;

use crate::config::config::ConfigRef;
use crate::fs::path::SPath;
use crate::tsw::lang_id::LangId;
use crate::w7e::handler::{Handler, NavCompRef};
use crate::w7e::handler_load_error::HandlerLoadError;

pub const INIT_TIMEOUT: Duration = Duration::from_millis(2000);

pub struct PythonHandler {
    root: SPath,

    navcomp: Option<NavCompRef>,
}

impl Handler for PythonHandler {
    fn lang_id(&self) -> LangId {
        LangId::PYTHON3
    }

    fn handler_id(&self) -> &'static str {
        "python3"
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
impl PythonHandler {
    pub fn load(_config: &ConfigRef, ff: SPath, navcomp_op: Option<NavCompRef>) -> Result<PythonHandler, HandlerLoadError> {
        if !ff.is_dir() {
            return Err(HandlerLoadError::NotAProject);
        }

        let compile_commands_file = ff.descendant_checked("requirements.txt").ok_or(HandlerLoadError::NotAProject)?;
        if !compile_commands_file.is_file() {
            return Err(HandlerLoadError::NotAProject);
        }

        // let contents = compile_commands_file.read_entire_file()?;

        Ok(PythonHandler {
            root: ff,
            navcomp: navcomp_op,
        })
    }
}
