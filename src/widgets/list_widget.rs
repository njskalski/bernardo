use std::fmt::Debug;

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

pub trait ListWidgetItem: Debug + Clone {
    //TODO change to static str?
    fn get_column_name(idx: usize) -> &'static str;
    fn get_min_column_width(idx: usize) -> u16;
    fn len_columns() -> usize;
    fn get(&self, idx: usize) -> Option<String>;
}

/*
    Keep the provider light
 */
pub trait ListWidgetProvider<Item: ListWidgetItem>: Debug {
    fn len(&self) -> usize;
    fn get(&self, idx: usize) -> Option<Item>;
}

struct ProviderIter<'a, Item: ListWidgetItem> {
    p: &'a dyn ListWidgetProvider<Item>,
    idx: usize,
}

impl<'a, LItem: ListWidgetItem> Iterator for ProviderIter<'a, LItem> {
    type Item = LItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.p.len() {
            None
        } else {
            let item = self.p.get(self.idx);
            self.idx += 1;
            item
        }
    }

    fn count(self) -> usize where Self: Sized {
        self.p.len()
    }
}

impl<Item: ListWidgetItem> dyn ListWidgetProvider<Item> {
    pub fn iter(&self) -> impl std::iter::Iterator<Item=Item> + '_ {
        ProviderIter {
            p: self,
            idx: 0,
        }
    }
}

impl<Item: ListWidgetItem> ListWidgetProvider<Item> for Vec<Item> {
    fn len(&self) -> usize {
        <[Item]>::len(self)
    }

    fn get(&self, idx: usize) -> Option<Item> {
        // // Vec::get(self, idx)
        // Some(self[idx].clone())
        // let self_as_vec: &Vec<Item> = self as &Vec<Item>;
        // self_as_vec.get(idx)
        <[Item]>::get(self, idx).map(|f| f.clone())
    }
}

pub struct ListWidget<Item: ListWidgetItem> {
    id: WID,

    provider: Box<dyn ListWidgetProvider<Item>>,
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
    Noop,
}

impl AnyMsg for ListWidgetMsg {}

impl<Item: ListWidgetItem> ListWidgetProvider<Item> for () {
    fn len(&self) -> usize {
        0
    }

    fn get(&self, _idx: usize) -> Option<Item> {
        None
    }
}

impl<Item: ListWidgetItem> ListWidget<Item> {
    pub fn new() -> Self {
        ListWidget {
            id: get_new_widget_id(),
            provider: Box::new(()),
            highlighted: None,
            show_column_names: true,
            on_miss: None,
            on_hit: None,
            on_change: None,
        }
    }

    pub fn with_provider(self, provider: Box<dyn ListWidgetProvider<Item>>) -> Self {
        ListWidget {
            provider,
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

    pub fn with_on_miss(self, on_miss: WidgetAction<Self>) -> Self {
        ListWidget {
            on_miss: Some(on_miss),
            ..self
        }
    }

    fn on_hit(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_hit.is_some() {
            self.on_hit.unwrap()(self)
        } else {
            None
        }
    }

    pub fn with_on_hit(self, on_hit: WidgetAction<Self>) -> Self {
        ListWidget {
            on_hit: Some(on_hit),
            ..self
        }
    }

    fn on_change(&self) -> Option<Box<dyn AnyMsg>> {
        if self.on_change.is_some() {
            self.on_change.unwrap()(self)
        } else {
            None
        }
    }

    pub fn with_on_change(self, on_change: WidgetAction<Self>) -> Self {
        ListWidget {
            on_change: Some(on_change),
            ..self
        }
    }

    pub fn set_provider(&mut self, provider: Box<dyn ListWidgetProvider<Item>>) {
        self.provider = provider
    }

    pub fn get_highlighted(&self) -> Option<Item> {
        self.highlighted.map(
            |idx| self.provider.get(idx)
        ).flatten()
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

        let from_items = self.min_size();
        let mut res = sc.visible_hint().size;

        if from_items.x > res.x && sc.x().is_none() {
            res.x = from_items.x;
        }

        if from_items.y > res.y && sc.y().is_none() {
            res.y = from_items.y;
        }

        res
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            InputEvent::KeyInput(key) => {
                match key.keycode {
                    Keycode::ArrowUp => {
                        if self.highlighted.map(|f| f > 0).unwrap_or(false) {
                            Some(ListWidgetMsg::Arrow(Arrow::Up))
                        } else {
                            None
                        }
                    }
                    Keycode::ArrowDown => {
                        if self.highlighted.map(|f| f + 1 < self.provider.len()).unwrap_or(false) {
                            Some(ListWidgetMsg::Arrow(Arrow::Down))
                        } else {
                            None
                        }
                    }
                    Keycode::ArrowLeft => {
                        None
                    }
                    Keycode::ArrowRight => {
                        None
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
        }.map(|m| Box::new(m) as Box<dyn AnyMsg>);
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
                            }
                            Arrow::Down => {
                                debug!("items {}, old_high {}", self.provider.len(), old_highlighted);
                                if old_highlighted + 1 < self.provider.len() {
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
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let primary_style = theme.default_text(focused);
        helpers::fill_output(primary_style.background, output);
        let cursor_style = theme.highlighted(focused);
        let header_style = theme.header(focused);

        // TODO add columns expansion
        // it's the same as in layouts, probably we should move that calc to primitives
        let mut y_offset: u16 = 0;

        if self.show_column_names {
            let mut x_offset: u16 = 0;
            for c_idx in 0..Item::len_columns() {
                output.print_at(
                    XY::new(x_offset, y_offset),
                    header_style,
                    Item::get_column_name(c_idx), // TODO cut agaist overflow
                );
                x_offset += Item::get_min_column_width(c_idx);
            }
            y_offset += 1;
        }

        for (idx, item) in self.provider.iter().enumerate() {
            debug!("y+idx = {}, osy = {:?}, item = {:?}", y_offset as usize + idx, output.size_constraint().y(), item);

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
                    Some(s) => s,
                    None => "".to_string(),
                };

                let column_width = Item::get_min_column_width(c_idx);

                output.print_at(
                    // TODO possible u16 overflow
                    // TODO handle overflow of column length
                    XY::new(x_offset, y_offset + idx as u16),
                    style,
                    &text,
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

