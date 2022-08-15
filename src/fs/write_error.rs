use std::io::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum WriteError {
    FileNotFound,
    UnmappedError(String),
}

impl From<std::io::Error> for WriteError {
    fn from(e: Error) -> Self {
        WriteError::UnmappedError(e.to_string())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WriteOrSerError {
    WriteError(WriteError),
    SerError(String),
}

impl From<WriteError> for WriteOrSerError {
    fn from(we: WriteError) -> Self {
        WriteOrSerError::WriteError(we)
    }
}

impl From<ron::Error> for WriteOrSerError {
    fn from(re: ron::Error) -> WriteOrSerError {
        WriteOrSerError::SerError(re.to_string())
    }
}