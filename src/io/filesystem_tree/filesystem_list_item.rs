use std::path::PathBuf;

use crate::widgets::list_widget::ListWidgetItem;

#[derive(Clone, Debug)]
pub struct FilesystemListItem {
    path: PathBuf,
}

impl FilesystemListItem {
    pub fn new(path: PathBuf) -> Self {
        FilesystemListItem {
            path
        }
    }
}

impl ListWidgetItem for FilesystemListItem {
    fn get_column_name(_idx: usize) -> &'static str {
        "name"
    }

    fn get_min_column_width(_idx: usize) -> u16 {
        10
    }

    fn len_columns() -> usize {
        1
    }

    fn get(&self, _idx: usize) -> Option<String> {
        self.path.to_str().map(|f| f.to_string())
    }
}