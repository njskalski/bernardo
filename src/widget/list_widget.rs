use crate::widget::widget::{Widget, WID, get_new_widget_id};
use crate::primitives::xy::XY;
use crate::io::input_event::InputEvent;
use crate::widget::any_msg::AnyMsg;
use crate::io::output::Output;
use crate::io::style::{TextStyle_WhiteOnBlue, TextStyle_WhiteOnBlack, TextStyle_WhiteOnBrightYellow};
use std::borrow::Borrow;

pub enum ListWidgetCell {
    NotAvailable,
    Loading,
    Ready(String),
}

pub trait ListWidgetItem {
    //TODO change to static str?
    fn get_column_name(idx: usize) -> String;
    fn get_min_column_width(idx: usize) -> u16;
    fn len_columns() -> usize;
    fn get(&self, idx: usize) -> ListWidgetCell;
}

pub struct ListWidget<Item: ListWidgetItem> {
    id: WID,
    // later probably change into some provider
    items: Vec<Item>,
    highlighted: Option<usize>,
    show_column_names: bool,
}

pub enum ListWidgetMsg {}

impl<Item: ListWidgetItem> ListWidget<Item> {
    pub fn new() -> Self {
        ListWidget {
            id: get_new_widget_id(),
            items: vec![],
            highlighted: None,
            show_column_names: true,
        }
    }

    pub fn with_items(self, items: Vec<Item>) -> Self {
        ListWidget {
            items,
            ..self
        }
    }

    pub fn with_selection(self) -> Self {
        ListWidget {
            highlighted: Some(0),
            ..self
        }
    }
}


impl<Item: ListWidgetItem> Widget for ListWidget<Item> {
    fn id(&self) -> WID {
        self.id
    }

    fn min_size(&self) -> XY {
        // completely arbitrary

        let rows = 2 + if self.show_column_names { 1 } else { 0 } as u16;
        let mut cols = 0;

        for i in 0..Item::len_columns() {
            cols += Item::get_min_column_width(i);
        }

        XY::new(rows, cols)
    }

    fn size(&self, max_size: XY) -> XY {
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn get_focused(&self) -> &dyn Widget {
        self
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        self
    }

    fn render(&self, focused: bool, output: &mut Output) {
        // TODO add columns expansion
        // it's the same as in layouts, probably we should move that calc to primitives
        let mut y_offset: u16 = 0;

        let header_style = TextStyle_WhiteOnBlue;
        let non_highlighted_style = TextStyle_WhiteOnBlack;
        let highlighted_style = TextStyle_WhiteOnBrightYellow;

        if self.show_column_names {
            let mut x_offset: u16 = 0;
            for c_idx in 0..Item::len_columns() {
                output.print_at(
                    XY::new(x_offset, y_offset),
                    header_style,
                    Item::get_column_name(c_idx).as_str(), // TODO cut agaist overflow
                );
                x_offset += Item::get_min_column_width(c_idx);
            }
            y_offset += 1;
        }

        for (idx, item) in self.items.iter().enumerate() {
            let mut x_offset: u16 = 0;

            let style = if self.highlighted == Some(idx) {
                highlighted_style
            } else {
                non_highlighted_style
            };

            for c_idx in 0..Item::len_columns() {
                let text: String = match item.get(c_idx) {
                    ListWidgetCell::NotAvailable => "N/A".to_string(),
                    ListWidgetCell::Loading => "...".to_string(),
                    ListWidgetCell::Ready(s) => s,
                };

                output.print_at(
                    // TODO possible u16 overflow
                    XY::new(x_offset, y_offset + idx as u16),
                    style,
                    text.as_str(),
                );

                x_offset += Item::get_min_column_width(c_idx);
            }
        }
    }
}

