use std::fmt::{Display, Formatter};
use std::str::Utf8Error;

use ron::Error;

#[derive(Debug)]
pub enum ReadError {
    IoError(std::io::Error),
    Utf8Error(Utf8Error),
    RonError(ron::Error),
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::str::Utf8Error> for ReadError {
    fn from(ue: Utf8Error) -> Self {
        ReadError::Utf8Error(ue)
    }
}

impl From<ron::Error> for ReadError {
    fn from(re: Error) -> Self {
        ReadError::RonError(re)
    }
}

impl From<std::io::Error> for ReadError {
    fn from(ie: std::io::Error) -> Self {
        ReadError::IoError(ie)
    }
}