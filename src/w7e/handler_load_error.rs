use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::fs::read_error::ReadError;

#[derive(Debug, Eq, PartialEq)]
pub enum HandlerLoadError {
    HandlerNotFound,
    NotAProject,
    ReadError(ReadError),
    DeserializationError(String),
}

impl From<ReadError> for HandlerLoadError {
    fn from(re: ReadError) -> Self {
        HandlerLoadError::ReadError(re)
    }
}

impl Display for HandlerLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}