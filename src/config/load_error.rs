// TODO: This file is very similar to ReadError in FS. Maybe it's worth merging them?
// They are however in one way distinct: we want to be able to Load Config from outside FS.

use std::fmt::{Display, Formatter};
use std::str::Utf8Error;

use crate::fs::read_error::ReadError;

#[derive(Debug)]
pub enum LoadError {
    ReadError(ReadError),
    IoError(std::io::Error),
    DeserializationError(ron::Error),
    UnmappedError(String),
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
        LoadError::ReadError(ReadError::from(ioe))
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

//TODO
impl From<ron::error::SpannedError> for LoadError {
    fn from(e: ron::error::SpannedError) -> Self {
        LoadError::UnmappedError(format!("{}", e))
    }
}

impl std::error::Error for LoadError {}