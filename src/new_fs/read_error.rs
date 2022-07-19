#[derive(Debug)]
pub enum ReadError {
    FileNotFound,
    NotAFilePath,
    DeError(ron::de::Error),
    UnmappedError(std::io::Error),
}

pub enum ListError {
    PathNotFound,
    UnmappedError(std::io::Error),
}