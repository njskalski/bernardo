use std::os::unix::raw::time_t;
use crate::widget::list_widget::{ListWidgetItem, ListWidgetCell};

struct MockFile {
    name: string,
    size: usize,
    filetype: Option<string>,
    // there should be date but I'm in the plane and have no docs.
}


impl ListWidgetItem for MockFile {
    fn len_columns() -> usize {
        3
    }

    fn get_column_name(idx: usize) -> String {
        match idx {
            0 => { "filename".to_string() }
            1 => { "size".to_string() }
            2 => { "type".to_string() }
            _ => { "N/A" }
        }
    }

    fn get(&self, idx: usize) -> ListWidgetCell {
        match idx {
            0 => { ListWidgetCell::Ready(self.name) }
            1 => { ListWidgetCell::Ready(format!("{}", self.size)) }
            2 => { ListWidgetCell::NotAvailable }
            _ => { ListWidgetCell::NotAvailable }
        }
    }
}

mod mock {
    use crate::widget::mock_file_list::MockFile;

    pub fn get_mock_file_list() -> Vec<MockFile> {
        vec![]
    }
}