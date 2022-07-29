use std::error::Error;

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