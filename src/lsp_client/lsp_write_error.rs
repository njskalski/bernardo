use std::io;

pub enum LspWriteError {
    WrongValueType,
    SerializationError(serde_json::error::Error),
    IoError(io::Error),
    BrokenPipe,
    InterruptedWrite,
}

impl From<serde_json::error::Error> for LspWriteError {
    fn from(e: serde_json::error::Error) -> Self {
        LspWriteError::SerializationError(e)
    }
}

impl From<io::Error> for LspWriteError {
    fn from(ioe: io::Error) -> Self {
        LspWriteError::IoError(ioe)
    }
}