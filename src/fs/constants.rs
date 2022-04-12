use std::path::Path;

pub const NON_UTF8_ERROR_STR: &'static str = "FAILED UTF-8 CAST";
pub const NOT_A_FILENAME: &'static str = "NOT A FILENAME";

pub fn is_sham(path: &Path) -> bool {
    for ancestor in path.ancestors() {
        if ancestor.file_name().map(|f| f.to_str()).flatten().map(|f| f.starts_with(".")).unwrap_or(false) {
            return true;
        }
    }

    false
}