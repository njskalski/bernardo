use std::io::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum ReadError {
    FileNotFound,
    NotAFilePath,
    // TODO separate?
    DeError(String),
    UnmappedError(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ListError {
    PathNotFound,
    NotADir,
    UnmappedError(String),
}

impl From<std::io::Error> for ReadError {
    fn from(e: Error) -> Self {
        ReadError::UnmappedError(e.to_string())
    }
}

impl From<std::io::Error> for ListError {
    fn from(e: Error) -> Self {
        ListError::UnmappedError(e.to_string())
    }
}

impl From<ron::de::Error> for ReadError {
    fn from(e: ron::Error) -> Self {
        ReadError::DeError(e.to_string())
    }
}