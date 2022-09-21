pub mod mock {
    use std::borrow::Cow;

    use crate::widgets::list_widget::list_widget_item::ListWidgetItem;

    #[derive(Clone, Debug)]
    pub struct MockFile {
        name: String,
        size: usize,
        filetype: Option<String>,
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

        fn get_column_name(idx: usize) -> &'static str {
            match idx {
                0 => { "filename" }
                1 => { "size" }
                2 => { "type" }
                _ => { "N/A" }
            }
        }

        fn get(&self, idx: usize) -> Option<Cow<'_, str>> {
            match idx {
                0 => { Some(Cow::Borrowed(&self.name)) }
                1 => { Some(Cow::Owned(format!("{}", self.size))) }
                _ => None
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