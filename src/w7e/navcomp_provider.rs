use std::fmt::Debug;

use crate::fs::path::SPath;
use crate::primitives::cursor_set::Cursor;

// this is a wrapper around LSP and "similar services".
pub trait NavCompProvider: Debug {
    /*
    file_contents are strictly LSP requirement
     */
    fn file_open_for_edition(&self, path: &SPath, file_contents: String);

    /*
    I will add "incremental updates" at later stage.
     */
    fn submit_edit_event(&self, path: &SPath, file_contents: String);

    fn completions(&self, path: &SPath, cursor: &Cursor);

    fn file_closed(&self, path: &SPath);
}