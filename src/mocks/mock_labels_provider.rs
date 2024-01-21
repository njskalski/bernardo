

use crate::fs::path::SPath;

use crate::widgets::editor_widget::label::label::Label;
use crate::widgets::editor_widget::label::labels_provider::LabelsProvider;

pub struct MockLabelsProvider {
    pub labels: Vec<Label>,
}

impl MockLabelsProvider {
    pub fn new() -> Self {
        MockLabelsProvider { labels: vec![] }
    }
}

impl LabelsProvider for MockLabelsProvider {
    fn query_for(&self, _path_op: Option<&SPath>) -> Box<dyn Iterator<Item = &'_ Label> + '_> {
        Box::new(self.labels.iter())
    }
}
