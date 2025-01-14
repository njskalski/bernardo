// This represents a highlighted symbol in a path.

use crate::cursor::cursor::Cursor;
use crate::fs::path::SPath;

#[derive(Debug)]
pub struct SymbolUsage {
    pub path: SPath,
    pub range: Cursor,
}
