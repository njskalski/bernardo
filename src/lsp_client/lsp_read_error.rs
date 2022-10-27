use std::io;
use std::sync::PoisonError;

use crossbeam_channel::RecvError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LspReadError {
    NoLine,
    NoContentLength,
    FromUtf8(String),
    IoError(String),
    DeError(String),
    UnknownMethod,
    ParamCastFailed,
    UnexpectedContents,
    NotSingleResponse,
    UnmatchedId { id: String, method: String },
    JsonRpcError(String),
    BrokenChannel,
    /*
    We retrieved response, but promise that was waiting for it has been dropped.
    TODO some day we should send cancellation in such cases
     */
    PromiseExpired { id: String },
    HttpParseError(String),
    LockError(String),
}

impl From<io::Error> for LspReadError {
    fn from(ioe: io::Error) -> Self {
        LspReadError::IoError(ioe.to_string())
    }
}

impl From<serde_json::error::Error> for LspReadError {
    fn from(dee: serde_json::error::Error) -> Self {
        LspReadError::DeError(dee.to_string())
    }
}

impl From<jsonrpc_core::Error> for LspReadError {
    fn from(_: jsonrpc_core::Error) -> Self {
        LspReadError::ParamCastFailed
    }
}

impl From<std::string::FromUtf8Error> for LspReadError {
    fn from(ue: std::string::FromUtf8Error) -> Self {
        LspReadError::FromUtf8(ue.to_string())
    }
}

impl<R> From<PoisonError<R>> for LspReadError {
    fn from(pe: PoisonError<R>) -> Self {
        LspReadError::LockError(pe.to_string())
    }
}

impl From<crossbeam_channel::RecvError> for LspReadError {
    fn from(_: RecvError) -> Self {
        LspReadError::BrokenChannel
    }
}