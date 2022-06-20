use std::io;

pub enum LspReadError {
    NoLine,
    IoError(io::Error),
    DeError(serde_json::error::Error),
    UnknownMethod,
    ParamCastFailed,
    UnexpectedContents,
}

impl From<io::Error> for LspReadError {
    fn from(ioe: io::Error) -> Self {
        LspReadError::IoError(ioe)
    }
}

impl From<serde_json::error::Error> for LspReadError {
    fn from(dee: serde_json::error::Error) -> Self {
        LspReadError::DeError(dee)
    }
}

impl From<jsonrpc_core::Error> for LspReadError {
    fn from(_: jsonrpc_core::Error) -> Self {
        LspReadError::ParamCastFailed
    }
}
