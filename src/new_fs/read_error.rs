pub enum ReadError {
    FileNotFound,
    UnmappedError(std::io::Error),
}

pub enum ListError {
    PathNotFound,
    UnmappedError(std::io::Error),
}