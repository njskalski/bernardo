use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::widgets::edit_box::{self, EditBoxWidget};
use crate::widgets::fuzzy_search::fuzzy_search::FuzzySearchWidget;

use super::editbox_interpreter::EditWidgetInterpreter;

pub struct FuzzySearchInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,

    editbox: EditWidgetInterpreter<'a>,
}

impl<'a> FuzzySearchInterpreter<'a> {
    pub fn new(output: &'a MetaOutputFrame, meta: &'a Metadata) -> Self {
        debug_assert!(meta.typename == FuzzySearchWidget::TYPENAME);

        let editbox: Vec<&Metadata> = output
            .get_meta_by_type(EditBoxWidget::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        assert_eq!(editbox.len(), 1);

        let editbox = EditWidgetInterpreter::new(editbox[0], output);

        Self { meta, output, editbox }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }

    pub fn contents(&self) -> String {
        self.output
            .buffer
            .lines_iter()
            .with_rect(self.meta.rect)
            .next()
            .unwrap()
            .text
            .trim()
            .to_string()
    }

    pub fn get_edit_box(&self) -> &'a EditWidgetInterpreter {
        &self.editbox
    }
}
