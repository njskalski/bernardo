use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum SaveError {
    SerializationError(ron::Error),
    IoError(std::io::Error),
}

impl From<ron::Error> for SaveError {
    fn from(e: ron::Error) -> Self {
        SaveError::SerializationError(e)
    }
}

impl From<std::io::Error> for SaveError {
    fn from(ioe: std::io::Error) -> Self {
        SaveError::IoError(ioe)
    }
}

impl Display for SaveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SaveError {}
