use crate::io::cell::Cell;
use crate::io::output::Metadata;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::primitives::xy::XY;
use crate::widgets::nested_menu::widget::{NESTED_MENU_FOLDER_CLOSED, NESTED_MENU_FOLDER_WIDHT};

pub struct NestedMenuInterpreter<'a> {
    meta: &'a Metadata,
    mock_output: &'a MetaOutputFrame,
}

#[derive(PartialEq, Eq)]
pub enum Status {
    Highlight,
    Expanded,
    Regular,
}

pub struct Item {
    pub label: String,
    pub depth: u16,
    pub leaf: bool,
    pub status: Status,
}

impl<'a> NestedMenuInterpreter<'a> {
    pub fn new(mock_output: &'a MetaOutputFrame, meta: &'a Metadata) -> Option<Self> {
        if let Some(item) = mock_output
            .get_meta_by_type(crate::widgets::nested_menu::widget::NESTED_MENU_TYPENAME)
            .next()
        {
            Some(NestedMenuInterpreter { meta, mock_output })
        } else {
            None
        }
    }

    // Returns only *drawn* items, and we skip drawing a lot of nodes that are not of user's interest.
    // TODO will silently fail with folders starting with "v"
    pub fn get_items(&self) -> impl Iterator<Item = Item> {
        let rect = self.meta.rect;
        let mut result: Vec<Item> = Default::default();

        for y in rect.upper_left().y..rect.lower_right().y {
            let mut first_letter_x = rect.upper_left().x;
            let mut item = Item {
                label: "".to_string(),
                depth: 0,
                leaf: true,
                status: Status::Highlight,
            };
            let mut first_letter_x = rect.upper_left().x;

            // figuring out the position of first letter and whether it's a folder or not
            for x in rect.upper_left().x..rect.lower_right().x {
                let xy = XY::new(x, y);

                match &self.mock_output.buffer[xy] {
                    Cell::Begin { style, grapheme } => {
                        if *grapheme == NESTED_MENU_FOLDER_CLOSED.to_string() || *grapheme == NESTED_MENU_FOLDER_CLOSED.to_string() {
                            debug_assert!((rect.upper_left().x - x) % NESTED_MENU_FOLDER_WIDHT == 0);
                            item.depth = (rect.upper_left().x - x) / NESTED_MENU_FOLDER_WIDHT;
                            item.leaf = false;
                        } else {
                            if grapheme != " " {
                                first_letter_x = x;
                                break;
                            }
                        }
                    }
                    Cell::Continuation => {}
                }
            }

            let mut items = self
                .mock_output
                .buffer
                .get_horizontal_piece(first_letter_x..rect.lower_right().x, y)
                .unwrap();
            let iter_item = items.next().unwrap();

            if iter_item.text.trim().is_empty() {
                break;
            }

            item.label = iter_item.text;

            let first_letter_style = *self.mock_output.buffer[XY::new(first_letter_x, y)].style().unwrap();

            let mut style_found = false;
            if first_letter_style == crate::widgets::nested_menu::widget::get_default_style(&self.mock_output.theme, true) {
                item.status = Status::Regular;
                style_found = true;
            }
            if first_letter_style == crate::widgets::nested_menu::widget::get_highlighted_style(&self.mock_output.theme, true) {
                debug_assert!(style_found == false);
                item.status = Status::Highlight;
                style_found = true;
            }
            if first_letter_style == crate::widgets::nested_menu::widget::get_expanded_style(&self.mock_output.theme, true) {
                debug_assert!(style_found == false);
                item.status = Status::Expanded;
                item.leaf = false;
                style_found = true;
            }
            debug_assert!(style_found);

            result.push(item);
        }

        result.into_iter()
    }

    pub fn get_selected_item(&self) -> Option<Item> {
        self.get_items().filter(|item| item.status == Status::Highlight).next()
    }
}
