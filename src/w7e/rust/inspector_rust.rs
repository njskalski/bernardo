use std::path::{Path, PathBuf};

use crate::fs::path::SPath;
use crate::LangId;
use crate::w7e::handler::Handler;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::inspector::LangInspector;
use crate::w7e::rust::handler_rust::RustHandler;

pub struct RustLangInspector {}

impl LangInspector for RustLangInspector {
    fn lang_id(&self) -> LangId {
        LangId::RUST
    }

    fn is_project_dir(&self, ff: &SPath) -> bool {
        ff.is_dir() && ff.descendant_checked("Cargo.toml").map(|desc| desc.is_file()).unwrap_or(false)
    }
}

impl RustLangInspector {
    pub fn new() -> Self {
        RustLangInspector {}
    }
}