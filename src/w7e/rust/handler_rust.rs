use std::path::{Path, PathBuf};
use crate::fs::file_front::FileFront;
use crate::LangId;
use crate::w7e::handler::Handler;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::handler_load_error::HandlerLoadError::ReadError;

pub struct RustHandler {
    root: FileFront,
    cargo_file: cargo_toml::Manifest,
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
}

impl RustHandler {
    pub fn load(ff: FileFront) -> Result<RustHandler, HandlerLoadError> {
        if !ff.is_dir() {
            return Err(HandlerLoadError::NotAProject);
        }

        let path_suffix = PathBuf::from("Cargo.toml");
        let cargo_file = ff.descendant(&path_suffix).ok_or(HandlerLoadError::NotAProject)?;
        if !cargo_file.is_file() {
            return Err(HandlerLoadError::NotAProject);
        }

        let contents = cargo_file.read_entire_file_to_bytes()?;
        let cargo = cargo_toml::Manifest::from_slice(&contents)
            .map_err(|e| HandlerLoadError::DeserializationError(Box::new(e)))?;

        Ok(RustHandler {
            root: ff,
            cargo_file: cargo,
        })
    }
}