use std::path::PathBuf;

use crate::widget::list_widget::{ListWidgetCell, ListWidgetItem};

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
    fn get_column_name(_idx: usize) -> String {
        "name".to_string()
    }

    fn get_min_column_width(_idx: usize) -> u16 {
        10
    }

    fn len_columns() -> usize {
        1
    }

    fn get(&self, _idx: usize) -> ListWidgetCell {
        // // TODO panic and looks like shit
        ListWidgetCell::Ready(self.path.file_name().unwrap().to_string_lossy().to_string())
    }
}