use std::io;
use std::sync::PoisonError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LspWriteError {
    WrongValueType,
    SerializationError(String),
    IoError(String),
    BrokenPipe,
    InterruptedWrite,
    LockError(String),
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

impl<R> From<PoisonError<R>> for LspWriteError {
    fn from(pe: PoisonError<R>) -> Self {
        LspWriteError::LockError(pe.to_string())
    }
}
