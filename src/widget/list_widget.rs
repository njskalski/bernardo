use std::fmt::Debug;
use std::iter::Iterator;
use std::slice::SliceIndex;

use log::{debug, warn};
use unicode_width::UnicodeWidthStr;

use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::primitives::arrow::Arrow;
use crate::primitives::helpers;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::theme::Theme;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
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
pub trait ListWidgetItem: Clone + Debug {
    //TODO change to static str?
    fn get_column_name(idx: usize) -> String;
    fn get_min_column_width(idx: usize) -> u16;
    fn len_columns() -> usize;
    fn get(&self, idx: usize) -> ListWidgetCell;
}

pub trait ListWidgetProvider<Item: ListWidgetItem> {
    fn len(&self) -> usize;
    fn get(&self, idx: usize) -> Option<&Item>;
}

impl<Item: ListWidgetItem> ListWidgetProvider<Item> for Vec<Item> {
    fn len(&self) -> usize {
        <[Item]>::len(self)
    }

    fn get(&self, idx: usize) -> Option<&Item> {
        // // Vec::get(self, idx)
        // Some(self[idx].clone())
        // let self_as_vec: &Vec<Item> = self as &Vec<Item>;
        // self_as_vec.get(idx)
        <[Item]>::get(self, idx)
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
    Noop
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
                Some(item) => self.items.push(item.clone()),
                None => {
                    warn!("ListWidget: failed unpacking provider item #{}", idx);
                }
            }
        }
    }

    pub fn set_items_it<T: Iterator<Item=Item>>(&mut self, provider: T) {
        self.items.clear();
        for c in provider {
            self.items.push(c);
        }
        debug!("new items {:?}", self.items)
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

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        debug_assert!(sc.bigger_equal_than(self.min_size()));

        // TODO check if items < max_u16
        let rows = (self.items.len() + if self.show_column_names { 1 } else { 0 }) as u16;
        let mut cols = 0;

        for i in 0..Item::len_columns() {
            cols += Item::get_min_column_width(i);
        }

        let desired = XY::new(cols, rows);

        debug!("layout, items.len = {}", self.items.len());

        desired.cut(sc)
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
                                debug!("items {}, old_high {}", self.items.len(), old_highlighted);
                                if old_highlighted + 1 < self.items.len() {
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
            ListWidgetMsg::Noop => { None }
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let primary_style = theme.default_text(focused).maybe_half(focused);
        helpers::fill_background(primary_style.background, output);
        let cursor_style = theme.cursor().maybe_half(focused);
        let header_style = theme.header().maybe_half(focused);

        // TODO add columns expansion
        // it's the same as in layouts, probably we should move that calc to primitives
        let mut y_offset: u16 = 0;

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
            debug!("y+idx = {}, osy = {:?}", y_offset as usize + idx, output.size_constraint().y());

            match output.size_constraint().y() {
                Some(y) => if y_offset as usize + idx >= y as usize {
                    break;
                }
                None => {}
            }

            let mut x_offset: u16 = 0;

            let style = if self.highlighted == Some(idx) {
                cursor_style
            } else {
                primary_style
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

