pub enum ReadError {
    FileNotFound,
    UnmappedError(std::io::Error),
}