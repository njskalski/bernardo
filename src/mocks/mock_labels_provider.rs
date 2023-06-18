use std::ops::Range;

use crate::fs::path::SPath;
use crate::text::text_buffer::TextBuffer;
use crate::widgets::editor_widget::label::label::Label;
use crate::widgets::editor_widget::label::labels_provider::LabelsProvider;

struct MockLabelsProvider {
    pub labels: Vec<Label>,
}

impl MockLabelsProvider {
    pub fn new() -> Self {
        MockLabelsProvider {
            labels: vec![]
        }
    }
}

impl LabelsProvider for MockLabelsProvider {
    fn query_for(&self, path_op: Option<&SPath>) -> Box<dyn Iterator<Item=&'_ Label> + '_> {
        Box::new(self.labels.iter())
    }
}

