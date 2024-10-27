// This represents a highlighted symbol in a path.

use crate::cursor::cursor::Cursor;
use crate::fs::path::SPath;

pub struct SymbolUsage {
    pub path: SPath,
    pub range: Cursor,
}
