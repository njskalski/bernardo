use crate::fs::file_front::FileFront;
use crate::LangId;
use crate::w7e::handler::Handler;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::inspector::{LangInspector};
use crate::w7e::rust::handler_rust::RustHandler;

pub struct RustLangInspector {}

impl LangInspector for RustLangInspector {
    fn lang_id(&self) -> LangId {
        LangId::RUST
    }

    fn is_project_dir(&self, ff: &FileFront) -> bool {
        ff.is_dir() && ff.descendant("Cargo.toml").map(|desc| desc.is_dir()).unwrap_or(false)
    }

    fn handle(&self, ff: FileFront) -> Result<Box<dyn Handler>, HandlerLoadError> {
        RustHandler::load(ff).map(|h| Box::new(h) as Box<dyn Handler>)
    }
}

impl RustLangInspector {
    pub fn new() -> Self {
        RustLangInspector {}
    }
}