use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::fuzzy_search::fuzzy_search::FuzzySearchWidget;

use super::editbox_interpreter::EditWidgetInterpreter;

pub struct FuzzySearchInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,

    editbox: EditWidgetInterpreter<'a>,

}

/// This type is used in fuzzy_file_open_test as if it was corresponding to FuzzyFileSearchWidget,
/// while in fact it does correspond to FuzzySearchWidget. It's not a bug, it's because
/// FuzzyFileSearchWidget does not introduce any new functionality at this time.
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

    pub fn highlighted(&self) -> Vec<String> {
        let editbox_rect = self.editbox.rect();
        let mut rect_without_editbox: Rect = self.meta.rect;
        rect_without_editbox.pos += XY::new(0, editbox_rect.size.y);
        rect_without_editbox.size.y -= editbox_rect.size.y;

        self.output.buffer.lines_iter().with_rect(rect_without_editbox)
            .filter(|line| line.text_style == Some(self.output.theme.highlighted(self.is_focused())))
            .map(|item| item.text.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect()
    }

    pub fn get_edit_box(&self) -> &'a EditWidgetInterpreter {
        &self.editbox
    }
}
