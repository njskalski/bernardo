use std::fmt::{Display, Formatter};
use std::str::Utf8Error;
use crate::fs::filesystem_front::ReadError;

#[derive(Debug)]
pub enum LoadError {
    ReadError(ReadError),
    DeserializationError(ron::Error),
}

impl From<ron::Error> for LoadError {
    fn from(e: ron::Error) -> Self {
        LoadError::DeserializationError(e)
    }
}

impl From<ReadError> for LoadError {
    fn from(re: ReadError) -> Self {
        LoadError::ReadError(re)
    }
}

impl From<std::io::Error> for LoadError {
    fn from(ioe: std::io::Error) -> Self {
        LoadError::ReadError(ReadError::IoError(ioe))
    }
}

impl From<std::str::Utf8Error> for LoadError {
    fn from(ue: Utf8Error) -> Self {
        LoadError::ReadError(ReadError::Utf8Error(ue))
    }
}

impl Display for LoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for LoadError {}