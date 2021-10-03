use std::os::unix::raw::time_t;
use crate::widget::list_widget::{ListWidgetItem, ListWidgetCell};

struct MockFile {
    name: string,
    size: usize,
    filetype: Option<string>,
    // there should be date but I'm in the plane and have no docs.
}

impl MockFile {
    pub fn new(name: String, size: usize) -> Self {
        MockFile {
            name,
            size,
            filetype: None,
        }
    }

    pub fn with_filetype(self, filetype: String) -> Self {
        MockFile {
            filetype: Some(filetype),
            ..self
        }
    }
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

    fn get_min_column_width(idx: usize) -> u16 {
        match idx {
            0 => 20,
            1 => 12,
            2 => 5,
            _ => 0,
        }
    }
}

mod mock {
    use crate::widget::mock_file_list::MockFile;

    pub fn get_mock_file_list() -> Vec<MockFile> {
        let mut res: Vec<MockFile> = vec![];
        for i in 0..10 {
            res.push(
                MockFile::new(format!("text_file_{}.txt", i), 1000 + i).with_filetype("txt".to_string())
            );
            res.push(
                MockFile::new(format!("photo_{}.png", i), 30000 + i).with_filetype("png".to_string())
            );
        }

        res
    }
}