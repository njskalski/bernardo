use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};

use log::{debug, warn};
use unicode_width::UnicodeWidthStr;

use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::io::style::{TextStyle_WhiteOnBlack, TextStyle_WhiteOnBlue, TextStyle_WhiteOnBrightYellow};
use crate::primitives::arrow::Arrow;
use crate::primitives::helpers;
use crate::primitives::theme::Theme;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::list_widget::ListWidgetMsg::Hit;
use crate::widget::widget::{get_new_widget_id, WID, Widget, WidgetAction};

pub enum ListWidgetCell {
    NotAvailable,
    Loading,
    Ready(String),
}

/*
    Items are held in Widget state itself, and overwritten with set_items when necessary. Therefore
    keep them lightweight as they are being cloned on update. Treat ListWidgetItem as "sub widget",
    a cached projection on actual data, and not the data itself. Do not store data in widgets!
 */
pub trait ListWidgetItem: Clone {
    //TODO change to static str?
    fn get_column_name(idx: usize) -> String;
    fn get_min_column_width(idx: usize) -> u16;
    fn len_columns() -> usize;
    fn get(&self, idx: usize) -> ListWidgetCell;
}

pub trait ListWidgetProvider<Item: ListWidgetItem> {
    fn len(&self) -> usize;
    fn get(&self, idx: usize) -> Option<Item>;
}

impl<Item: ListWidgetItem> ListWidgetProvider<Item> for Vec<Item> {
    fn len(&self) -> usize {
        self.len()
    }

    fn get(&self, idx: usize) -> Option<Item> {
        self.get(idx).clone()
    }
}

pub struct ListWidget<Item: ListWidgetItem> {
    id: WID,
    // later probably change into some provider
    items: Vec<Item>,
    highlighted: Option<usize>,
    show_column_names: bool,

    on_hit: Option<WidgetAction<Self>>,
    on_change: Option<WidgetAction<Self>>,
    // miss is trying to make illegal move. Like backspace on empty, left on leftmost etc.
    on_miss: Option<WidgetAction<Self>>,
}

#[derive(Clone, Copy, Debug)]
pub enum ListWidgetMsg {
    Arrow(Arrow),
    Hit,
    Home,
    End,
    PageUp,
    PageDown,
}

impl AnyMsg for ListWidgetMsg {}

impl<Item: ListWidgetItem> ListWidget<Item> {
    pub fn new() -> Self {
        ListWidget {
            id: get_new_widget_id(),
            items: vec![],
            highlighted: None,
            show_column_names: true,
            on_miss: None,
            on_hit: None,
            on_change: None,
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

    fn on_miss(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_miss.is_some() {
            self.on_miss.unwrap()(self)
        } else {
            None
        }
    }

    fn on_hit(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_hit.is_some() {
            self.on_hit.unwrap()(self)
        } else {
            None
        }
    }

    fn on_change(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_change.is_some() {
            self.on_change.unwrap()(self)
        } else {
            None
        }
    }

    pub fn set_items(&mut self, provider: &mut dyn ListWidgetProvider<Item>) {
        self.items.clear();
        for idx in 0..provider.len() {
            match provider.get(idx) {
                Some(item) => self.items.push(item),
                None => {
                    warn!("ListWidget: failed unpacking provider item #{}", idx);
                }
            }
        }
    }
}


impl<Item: ListWidgetItem> Widget for ListWidget<Item> {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        "List"
    }

    fn min_size(&self) -> XY {
        // completely arbitrary

        let rows = 2 + if self.show_column_names { 1 } else { 0 } as u16;
        let mut cols = 0;

        for i in 0..Item::len_columns() {
            cols += Item::get_min_column_width(i);
        }

        XY::new(cols, rows)
    }

    fn layout(&mut self, max_size: XY) -> XY {
        debug_assert!(self.min_size().x <= max_size.x, "min_size {} max_size {}", self.min_size(), max_size);
        debug_assert!(self.min_size().y <= max_size.y, "min_size {} max_size {}", self.min_size(), max_size);

        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            InputEvent::KeyInput(key) => {
                match key.keycode {
                    Keycode::ArrowUp => {
                        Some(ListWidgetMsg::Arrow(Arrow::Up))
                    }
                    Keycode::ArrowDown => {
                        Some(ListWidgetMsg::Arrow(Arrow::Down))
                    }
                    Keycode::ArrowLeft => {
                        Some(ListWidgetMsg::Arrow(Arrow::Left))
                    }
                    Keycode::ArrowRight => {
                        Some(ListWidgetMsg::Arrow(Arrow::Right))
                    }
                    Keycode::Enter => {
                        Some(ListWidgetMsg::Hit)
                    }
                    Keycode::Home => {
                        Some(ListWidgetMsg::Home)
                    }
                    Keycode::End => {
                        Some(ListWidgetMsg::End)
                    }
                    Keycode::PageUp => {
                        Some(ListWidgetMsg::PageUp)
                    }
                    Keycode::PageDown => {
                        Some(ListWidgetMsg::PageDown)
                    }
                    _ => None
                }
            }
            _ => None
        }.map(|m| Box::new(m) as Box<dyn AnyMsg>)
    }


    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<ListWidgetMsg>();
        if our_msg.is_none() {
            warn!("expecetd ListWidgetMsg, got {:?}", msg);
            return None;
        }

        return match our_msg.unwrap() {
            ListWidgetMsg::Arrow(arrow) => {
                match self.highlighted {
                    None => None,
                    Some(old_highlighted) => {
                        match arrow {
                            Arrow::Up => {
                                if old_highlighted > 0 {
                                    self.highlighted = Some(old_highlighted - 1);
                                    self.on_change()
                                } else {
                                    self.on_miss()
                                }
                            },
                            Arrow::Down => {
                                if old_highlighted < self.items.len() - 1 {
                                    self.highlighted = Some(old_highlighted + 1);
                                    self.on_change()
                                } else {
                                    self.on_miss()
                                }
                            }
                            Arrow::Left => { None }
                            Arrow::Right => { None }
                        }
                    }
                }
            }
            ListWidgetMsg::Hit => {
                self.on_hit()
            }
            ListWidgetMsg::Home => { None }
            ListWidgetMsg::End => { None }
            ListWidgetMsg::PageUp => { None }
            ListWidgetMsg::PageDown => { None }
        }
    }

    fn get_focused(&self) -> &dyn Widget {
        self
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        self
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut Output) {
        let bgcolor = if focused {
            theme.active_edit().background
        } else {
            theme.inactive_edit().background
        };

        helpers::fill_background(bgcolor, output);

        // TODO add columns expansion
        // it's the same as in layouts, probably we should move that calc to primitives
        let mut y_offset: u16 = 0;

        let header_style = theme.header();

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
            // debug!("y+idx = {}, osy = {}", y_offset as usize + idx, output.size().y);
            if y_offset as usize + idx >= output.size().y as usize {
                break;
            }

            let mut x_offset: u16 = 0;

            let style = if self.highlighted == Some(idx) {
                if focused {
                    theme.active_cursor()
                } else {
                    theme.inactive_edit()
                }
            } else {
                theme.non_interactive_text(focused)
            };

            for c_idx in 0..Item::len_columns() {
                let text: String = match item.get(c_idx) {
                    ListWidgetCell::NotAvailable => "N/A".to_string(),
                    ListWidgetCell::Loading => "...".to_string(),
                    ListWidgetCell::Ready(s) => s,
                };

                let column_width = Item::get_min_column_width(c_idx);

                output.print_at(
                    // TODO possible u16 overflow
                    // TODO handle overflow of column length
                    XY::new(x_offset, y_offset + idx as u16),
                    style,
                    text.as_str(),
                );

                if text.width() < column_width as usize {
                    // since text.width() is < column_width, it's safe to cast to u16.
                    for x_stride in (text.width() as u16)..column_width {
                        let pos = XY::new(x_offset + x_stride, y_offset + idx as u16);
                        // debug!("printing at pos {} size {}", pos, output.size());
                        output.print_at(
                            // TODO possible u16 oveflow
                            pos,
                            style,
                            " ",
                        );
                    }
                }

                x_offset += Item::get_min_column_width(c_idx);
            }
        }
    }
}

