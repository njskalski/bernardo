use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;

pub struct NestedMenuInterpreter<'a> {
    meta: &'a Metadata,
    mock_output: &'a MetaOutputFrame,
}

pub struct SelectedItem {
    pub label : String,
    pub leaf : bool,
}

impl<'a> NestedMenuInterpreter<'a> {
    pub fn new(mock_output: &'a MetaOutputFrame, meta: &'a Metadata) -> Option<Self> {
        if let Some(item) = mock_output.get_meta_by_type(crate::widgets::nested_menu::widget::NESTED_MENU_TYPENAME).next() {
            Some(NestedMenuInterpreter{
                meta,
                mock_output,
            })
        } else {
            None
        }
    }

    pub fn get_selected_item(&self) -> SelectedItem {
        for y in self.meta.rect.upper_left().y..self.meta.rect.lower_right().y {
            // TODO
        }
    }
}
