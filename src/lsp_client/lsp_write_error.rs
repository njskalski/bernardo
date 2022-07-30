use std::io;

#[derive(Debug, Eq, PartialEq)]
pub enum LspWriteError {
    WrongValueType,
    SerializationError(String),
    IoError(String),
    BrokenPipe,
    InterruptedWrite,
}

impl From<serde_json::error::Error> for LspWriteError {
    fn from(e: serde_json::error::Error) -> Self {
        LspWriteError::SerializationError(e.to_string())
    }
}

impl From<io::Error> for LspWriteError {
    fn from(ioe: io::Error) -> Self {
        LspWriteError::IoError(ioe.to_string())
    }
}