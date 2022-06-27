use crate::lsp_client::lsp_read_error::LspReadError;
use crate::lsp_client::lsp_write_error::LspWriteError;

#[derive(Debug)]
pub enum LspIOError {
    Write(LspWriteError),
    Read(LspReadError),
}

impl From<LspReadError> for LspIOError {
    fn from(r: LspReadError) -> Self {
        LspIOError::Read(r)
    }
}

impl From<LspWriteError> for LspIOError {
    fn from(w: LspWriteError) -> Self {
        LspIOError::Write(w)
    }
}