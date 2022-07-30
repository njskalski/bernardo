use std::io;

#[derive(Debug, Eq, PartialEq)]
pub enum LspReadError {
    NoLine,
    IoError(String),
    DeError(String),
    UnknownMethod,
    ParamCastFailed,
    UnexpectedContents,
    NotSingleResponse,
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

impl From<stream_httparse::streaming_parser::ParseError> for LspReadError {
    fn from(p: stream_httparse::streaming_parser::ParseError) -> Self {
        LspReadError::HttpParseError(p.to_string())
    }
}