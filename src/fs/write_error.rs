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

// impl From<std::io::Error> for ListError {
//     fn from(e: Error) -> Self {
//         ListError::UnmappedError(e.to_string())
//     }
// }
//
// impl From<ron::de::Error> for ReadError {
//     fn from(e: ron::Error) -> Self {
//         ReadError::DeError(e.to_string())
//     }
// }