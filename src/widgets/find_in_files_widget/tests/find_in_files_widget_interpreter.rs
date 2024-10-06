use crate::io::output::Metadata;
use crate::mocks::button_interpreter::ButtonWidgetInterpreter;
use crate::mocks::editbox_interpreter::EditWidgetInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::primitives::rect::Rect;
use crate::widget::widget::Widget;
use crate::widgets::button::ButtonWidget;
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::find_in_files_widget::find_in_files_widget::FindInFilesWidget;

pub struct FindInFilesWidgetInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,
}

impl<'a> FindInFilesWidgetInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        debug_assert!(meta.typename == FindInFilesWidget::static_typename());

        Self { meta, output }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }

    pub fn rect(&self) -> Rect {
        self.meta.rect
    }

    pub fn query_box(&self) -> EditWidgetInterpreter<'_> {
        let meta = self.output.get_meta_by_type(EditBoxWidget::static_typename()).next().unwrap();

        EditWidgetInterpreter::new(meta, self.output)
    }

    pub fn filter_box(&self) -> EditWidgetInterpreter<'_> {
        let meta = self
            .output
            .get_meta_by_type(EditBoxWidget::static_typename())
            .skip(1)
            .next()
            .unwrap();

        EditWidgetInterpreter::new(meta, self.output)
    }

    pub fn cancel_button(&self) -> ButtonWidgetInterpreter<'_> {
        let meta = self
            .output
            .get_meta_by_type(ButtonWidget::static_typename())
            .skip(1)
            .next()
            .unwrap();
        let interp = ButtonWidgetInterpreter::new(meta, self.output);

        debug_assert!(interp.contents().contains("Cancel"));

        interp
    }

    pub fn search_button(&self) -> ButtonWidgetInterpreter<'_> {
        let meta = self.output.get_meta_by_type(ButtonWidget::static_typename()).next().unwrap();
        let interp = ButtonWidgetInterpreter::new(meta, self.output);

        debug_assert!(interp.contents().contains("Search"));

        interp
    }
}
