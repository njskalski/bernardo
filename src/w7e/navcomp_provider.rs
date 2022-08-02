use std::fmt::Debug;

use crate::fs::path::SPath;

// this is a wrapper around LSP and "similar services".
pub trait NavCompProvider: Debug {
    /*
    file_contents are strictly LSP requirement
     */
    fn file_open_for_edition(&self, path: &SPath, file_contents: String);
    fn file_closed(&self, path: &SPath);
}