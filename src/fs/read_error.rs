use std::fmt::{Display, Formatter};
use std::io::Error;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

#[derive(Debug, PartialEq, Eq)]
pub enum ReadError {
    FileNotFound,
    NotAFilePath,
    // TODO separate?
    DeError(String),
    Utf8Error(std::str::Utf8Error),
    UnmappedError(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ListError {
    PathNotFound,
    NotADir,
    UnmappedError(String),
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO something smarter?
        write!(f, "{:?}", self)
    }
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

impl From<Utf8Error> for ReadError {
    fn from(ue: Utf8Error) -> Self {
        ReadError::Utf8Error(ue)
    }
}

impl From<FromUtf8Error> for ReadError {
    fn from(fue: FromUtf8Error) -> Self {
        ReadError::Utf8Error(fue.utf8_error())
    }
}