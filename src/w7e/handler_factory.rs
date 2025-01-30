use std::sync::Arc;

use log::{debug, error};

use crate::config::config::ConfigRef;
use crate::experiments::buffer_register::BufferRegisterRef;
use crate::fs::path::SPath;
use crate::tsw::lang_id::LangId;
use crate::w7e::cpp::handler_cpp::CppHandler;
use crate::w7e::golang::handler_golang::GolangHandler;
use crate::w7e::handler::{Handler, NavCompRef};
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_group::NavCompTickSender;
use crate::w7e::navcomp_provider_lsp::NavCompProviderLsp;
use crate::w7e::python::handler_python::PythonHandler;
use crate::w7e::rust::handler_rust::RustHandler;

/*
This is a single point of entry to loading LanguageHandlers, to be used by both workspace generator
    and deserializer
 */

// TODO move lsp_path to workspace? Or at least allow override.
pub fn handler_factory(
    config: &ConfigRef,
    buffer_register_ref: &BufferRegisterRef,
    handler_id: &str,
    ff: SPath,
    navcomp_tick_sender: NavCompTickSender,
) -> Result<Box<dyn Handler>, HandlerLoadError> {
    debug!("attempting to load handler {} for {:?}", handler_id, ff.absolute_path());
    match handler_id {
        "rust" => {
            //So handler can "partially work", meaning for instance that running/debugging works, but LSP does not.
            let lsp_path = config.global.get_rust_lsp_path().ok_or(HandlerLoadError::LspNotFound)?;
            let workspace_root = ff.absolute_path();
            let mut navcomp_op: Option<NavCompRef> = None;
            if let Some(navcomp_lsp) = NavCompProviderLsp::new(lsp_path, workspace_root, LangId::RUST, navcomp_tick_sender) {
                navcomp_op = Some(NavCompRef::new(Box::new(navcomp_lsp)));
            } else {
                error!("LspWrapper construction failed.")
            }

            match RustHandler::load(config, ff, navcomp_op) {
                Ok(o) => Ok(Box::new(o)),
                Err(e) => Err(e),
            }
        }
        "cpp" => {
            //So handler can "partially work", meaning for instance that running/debugging works, but LSP does not.

            let lsp_path = config.global.get_clangd_lsp_path().ok_or(HandlerLoadError::LspNotFound)?;
            let workspace_root = ff.absolute_path();
            let mut navcomp_op: Option<NavCompRef> = None;
            if let Some(navcomp_lsp) = NavCompProviderLsp::new(lsp_path, workspace_root, LangId::CPP, navcomp_tick_sender) {
                navcomp_op = Some(NavCompRef::new(Box::new(navcomp_lsp)));
            } else {
                error!("LspWrapper construction failed.")
            }

            match CppHandler::load(config, ff, navcomp_op) {
                Ok(o) => Ok(Box::new(o)),
                Err(e) => Err(e),
            }
        }
        "go" => {
            let lsp_path = config.global.get_golang_lsp_path().ok_or(HandlerLoadError::LspNotFound)?;
            let workspace_root = ff.absolute_path();
            let mut navcomp_op: Option<NavCompRef> = None;
            if let Some(navcomp_lsp) = NavCompProviderLsp::new(lsp_path, workspace_root, LangId::GO, navcomp_tick_sender) {
                navcomp_op = Some(NavCompRef::new(Box::new(navcomp_lsp)));
            } else {
                error!("LspWrapper construction failed.")
            }

            match GolangHandler::load(config, ff, navcomp_op) {
                Ok(o) => Ok(Box::new(o)),
                Err(e) => Err(e),
            }
        }
        "python3" => {
            let lsp_path = config.global.get_python_lsp_path().ok_or(HandlerLoadError::LspNotFound)?;
            let workspace_root = ff.absolute_path();
            let mut navcomp_op: Option<NavCompRef> = None;
            if let Some(navcomp_lsp) = NavCompProviderLsp::new(lsp_path, workspace_root, LangId::PYTHON3, navcomp_tick_sender) {
                navcomp_op = Some(NavCompRef::new(Box::new(navcomp_lsp)));
            } else {
                error!("LspWrapper construction failed.")
            }

            match PythonHandler::load(config, ff, navcomp_op) {
                Ok(o) => Ok(Box::new(o)),
                Err(e) => Err(e),
            }
        }

        _ => Err(HandlerLoadError::HandlerNotFound),
    }
}
