use std::io;

use ron::Error;

#[derive(Debug)]
pub enum WriteError {
    RonSeError(ron::Error),
    IoError(io::Error),
}

impl From<ron::Error> for WriteError {
    fn from(se: Error) -> Self {
        WriteError::RonSeError(se)
    }
}

impl From<io::Error> for WriteError {
    fn from(ie: io::Error) -> Self {
        WriteError::IoError(ie)
    }
}