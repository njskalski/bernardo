use std::io;

#[derive(Debug, Eq, PartialEq)]
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
    HttpParseError(String),
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