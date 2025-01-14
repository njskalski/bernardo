use std::fmt::{Display, Formatter};

use crate::fs::read_error::ReadError;
use crate::lsp_client::lsp_io_error::LspIOError;

#[derive(Debug, Eq, PartialEq)]
pub enum HandlerLoadError {
    NoHandlerId,
    HandlerNotFound,
    HandlerNotAvailable,
    NotAProject,
    LspNotFound,
    ReadError(ReadError),
    DeserializationError(String),
    LspConstructionError,
    LspIOError(LspIOError),
    LspTimeout,
}

impl From<ReadError> for HandlerLoadError {
    fn from(re: ReadError) -> Self {
        HandlerLoadError::ReadError(re)
    }
}

impl Display for HandlerLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
