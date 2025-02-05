use crate::io::output::Metadata;
use crate::mocks::button_interpreter::ButtonWidgetInterpreter;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::mocks::text_widget_interpreter::TextWidgetInterpreter;
use crate::widgets::button::ButtonWidget;
use crate::widgets::generic_dialog::generic_dialog::GenericDialog;
use crate::widgets::text_widget::TextWidget;

#[derive(Clone, Debug)]
pub struct GenericDialogWidgetInterpreter<'a> {
    meta: &'a Metadata,
    output: &'a MetaOutputFrame,

    text_interpreter: TextWidgetInterpreter<'a>,

    buttons: Vec<ButtonWidgetInterpreter<'a>>,
}

impl<'a> GenericDialogWidgetInterpreter<'a> {
    pub fn new(meta: &'a Metadata, output: &'a MetaOutputFrame) -> Self {
        debug_assert!(meta.typename == GenericDialog::TYPENAME);

        let text_meta: Vec<&Metadata> = output
            .get_meta_by_type(TextWidget::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        assert_eq!(text_meta.len(), 1);

        let text_interpreter = TextWidgetInterpreter::new(text_meta[0], output);

        let button_metas: Vec<&Metadata> = output
            .get_meta_by_type(ButtonWidget::TYPENAME)
            .filter(|c| meta.rect.contains_rect(c.rect))
            .collect();

        let buttons: Vec<_> = button_metas.into_iter().map(|m| ButtonWidgetInterpreter::new(m, output)).collect();

        Self {
            meta,
            output,
            text_interpreter,
            buttons,
        }
    }

    pub fn is_focused(&self) -> bool {
        self.meta.focused
    }

    pub fn get_text(&self) -> &TextWidgetInterpreter<'a> {
        &self.text_interpreter
    }

    pub fn get_buttons(&self) -> &Vec<ButtonWidgetInterpreter<'a>> {
        &self.buttons
    }

    pub fn get_button_by_text(&self, text: &str) -> Option<&ButtonWidgetInterpreter<'a>> {
        for item in self.buttons.iter() {
            if item.contents().contains(text) {
                return Some(item);
            }
        }
        None
    }
}
