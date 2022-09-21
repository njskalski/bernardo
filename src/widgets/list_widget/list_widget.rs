use std::any::Any;
use std::cmp::min;
use std::fmt::Debug;
use std::string::String;

use log::{debug, error, warn};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::primitives::arrow::Arrow;
use crate::primitives::common_query::CommonQuery;
use crate::primitives::helpers;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::fill_policy::FillPolicy;
use crate::widget::widget::{get_new_widget_id, WID, Widget, WidgetAction};
use crate::widgets::list_widget::list_widget_item::ListWidgetItem;
use crate::widgets::list_widget::list_widget_provider::ListWidgetProvider;

pub struct ListWidget<Item: ListWidgetItem> {
    id: WID,

    provider: Box<dyn ListWidgetProvider<Item>>,
    highlighted: Option<usize>,
    show_column_names: bool,

    on_hit: Option<WidgetAction<Self>>,
    on_change: Option<WidgetAction<Self>>,
    // miss is trying to make illegal move. Like backspace on empty, left on leftmost etc.
    on_miss: Option<WidgetAction<Self>>,

    query: Option<CommonQuery>,

    fill_policy: FillPolicy,

    last_size: Option<XY>,
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

    fn get(&self, _idx: usize) -> Option<&Item> {
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
            last_size: None,
            fill_policy: Default::default(),
            query: None,
        }
    }

    pub fn with_provider(self, provider: Box<dyn ListWidgetProvider<Item>>) -> Self {
        ListWidget {
            provider,
            ..self
        }
    }

    pub fn items(&self) -> impl Iterator<Item=&Item> {
        self.provider.iter()
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

    pub fn get_highlighted(&self) -> Option<&Item> {
        self.highlighted.map(
            |idx| self.provider.get(idx)
        ).flatten()
    }

    pub fn set_show_column_names(&mut self, show_column_names: bool) {
        self.show_column_names = show_column_names;
    }

    pub fn with_show_column_names(self, show_column_names: bool) -> Self {
        ListWidget {
            show_column_names,
            ..self
        }
    }

    pub fn set_highlighted(&mut self, highlighted: usize) -> bool {
        if self.items().count() >= highlighted {
            self.highlighted = Some(highlighted);
            true
        } else {
            false
        }
    }

    pub fn with_fill_policy(self, fill_policy: FillPolicy) -> Self {
        Self {
            fill_policy,
            ..self
        }
    }

    pub fn set_fill_policy(&mut self, fill_policy: FillPolicy) {
        self.fill_policy = fill_policy;
    }

    pub fn get_fill_policy(&self) -> FillPolicy {
        self.fill_policy
    }

    pub fn get_provider(&self) -> &dyn ListWidgetProvider<Item> {
        self.provider.as_ref()
    }

    pub fn get_provider_mut(&mut self) -> &mut dyn ListWidgetProvider<Item> {
        self.provider.as_mut()
    }

    pub fn with_query(self, query: CommonQuery) -> Self {
        Self {
            query: Some(query),
            ..self
        }
    }

    pub fn set_query(&mut self, query: Option<CommonQuery>) {
        self.query = query;
    }

    pub fn get_query(&self) -> Option<&CommonQuery> {
        self.query.as_ref()
    }
}


impl<Item: ListWidgetItem + 'static> Widget for ListWidget<Item> {
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

    fn update_and_layout(&mut self, sc: SizeConstraint) -> XY {
        debug_assert!(sc.bigger_equal_than(self.min_size()),
                      "sc: {} self.min_size(): {}",
                      sc, self.min_size());

        let from_items = self.min_size();
        let mut res = sc.visible_hint().size;

        if from_items.x > res.x && sc.x().is_none() {
            res.x = from_items.x;
        }

        if from_items.y > res.y && sc.y().is_none() {
            res.y = from_items.y;
        }

        let res = self.fill_policy.get_size_from_constraints(&sc, res);
        self.last_size = Some(res);

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
        let size = if self.last_size.is_none() {
            error!("request to draw before layout, skipping.");
            return;
        } else {
            self.last_size.unwrap()
        };

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
                // TODO adjust lenght for the last column
                x_offset += Item::get_min_column_width(c_idx);
            }
            y_offset += 1;
        }

        for (idx, item) in self.provider.iter().enumerate() {
            debug!("y+idx = {}, osy = {:?}, item = {:?}", y_offset as usize + idx, output.size_constraint().y(), item);

            if y_offset as usize + idx >= size.y as usize {
                break;
            }

            let mut x_offset: u16 = 0;

            let style = if self.highlighted == Some(idx) {
                cursor_style
            } else {
                primary_style
            };

            for c_idx in 0..Item::len_columns() {
                let column_width = Item::get_min_column_width(c_idx);

                if x_offset >= size.x {
                    warn!("completely skipping drawing a column {} and consecutive, because it's offset is beyond output", c_idx);
                    break;
                }

                let mut max_x = min(size.x - x_offset, Item::get_min_column_width(c_idx));

                // TODO implement proper dividing space between columns when fill_policy.fill_x == true
                // If it's a last column and we are supposed to fill_x
                if c_idx + 1 == Item::len_columns() && self.fill_policy.fill_x {
                    max_x = size.x - x_offset;
                }

                // at this point it's safe to substract
                let actual_max_text_length = max_x - x_offset;
                debug_assert!(actual_max_text_length > 0);

                // huge block to "cut th text if necessary"
                let mut text = {
                    let full_text = match item.get(c_idx) {
                        Some(s) => s,
                        None => "".to_string(),
                    };
                    if full_text.len() <= actual_max_text_length as usize {
                        full_text
                    } else {
                        let mut cut_text = String::new();
                        let mut used_width: usize = 0;
                        for grapheme in full_text.graphemes(true) {
                            let grapheme_width = grapheme.width();
                            if grapheme_width + used_width > actual_max_text_length as usize {
                                break;
                            } else {
                                cut_text += grapheme;
                                used_width += grapheme_width;
                            }
                        }
                        cut_text
                    }
                };
                debug_assert!(text.width() <= actual_max_text_length as usize);

                output.print_at(
                    // TODO possible u16 overflow
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

