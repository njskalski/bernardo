use std::error::Error;
use crate::fs::filesystem_front::ReadError;

#[derive(Debug)]
pub enum HandlerLoadError {
    HandlerNotFound,
    NotAProject,
    ReadError(ReadError),
    DeserializationError(Box<dyn Error>),
}

impl From<ReadError> for HandlerLoadError {
    fn from(re: ReadError) -> Self {
        HandlerLoadError::ReadError(re)
    }
}