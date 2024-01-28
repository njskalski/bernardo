use std::cmp::min;
use std::fmt::Debug;
use std::string::String;

use log::{debug, error, warn};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::primitives::arrow::Arrow;
use crate::primitives::common_query::CommonQuery;
use crate::primitives::helpers;
use crate::primitives::helpers::copy_first_n_columns;
use crate::primitives::xy::XY;
use crate::unpack_or_e;
use crate::widget::any_msg::AnyMsg;
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WidgetAction, WID};
use crate::widgets::list_widget::list_widget_item::ListWidgetItem;
use crate::widgets::list_widget::provider::ListItemProvider;

// TODO there is an issue here. Theoretically between prelayout and render, the number of items in
// provider can SHORTEN making "highlighted" invalid. This highlights very clearly the problem with
// "providers" - they are not guaranteed to be constant. I'd have to copy the data from them out, or
// introduce some new invariants.

pub const TYPENAME: &str = "list_widget";

pub struct ListWidget<Item: ListWidgetItem> {
    id: WID,

    provider: Box<dyn ListItemProvider<Item>>,
    highlighted: Option<usize>,
    show_column_names: bool,

    on_hit: Option<WidgetAction<Self>>,
    on_change: Option<WidgetAction<Self>>,
    // miss is trying to make illegal move. Like backspace on empty, left on leftmost etc.
    on_miss: Option<WidgetAction<Self>>,

    query: Option<CommonQuery>,

    fill_policy: SizePolicy,

    last_size: Option<Screenspace>,
}

// TODO reduce with ScrollEnum
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
            provider: Box::new(()),
            highlighted: None,
            show_column_names: true,
            on_miss: None,
            on_hit: None,
            on_change: None,
            fill_policy: Default::default(),
            query: None,
            last_size: None,
        }
    }

    pub fn with_provider(self, provider: Box<dyn ListItemProvider<Item>>) -> Self {
        ListWidget { provider, ..self }
    }

    pub fn items(&self) -> Box<dyn Iterator<Item = &Item> + '_> {
        if let Some(query) = self.query.as_ref() {
            Box::new(
                self.provider
                    .items()
                    .filter(|item| item.get(0).map(|value| query.matches(&value)).unwrap_or(false)),
            )
        } else {
            Box::new(self.provider.items())
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

    pub fn set_provider(&mut self, provider: Box<dyn ListItemProvider<Item> + 'static>) {
        self.provider = provider
    }

    pub fn get_provider(&self) -> &dyn ListItemProvider<Item> {
        self.provider.as_ref()
    }

    pub fn get_provider_mut(&mut self) -> &mut dyn ListItemProvider<Item> {
        self.provider.as_mut()
    }

    pub fn get_highlighted(&self) -> Option<&Item> {
        self.highlighted.map(|idx| self.provider.items().nth(idx)).flatten()
    }

    pub fn set_show_column_names(&mut self, show_column_names: bool) {
        self.show_column_names = show_column_names;
    }

    pub fn with_show_column_names(self, show_column_names: bool) -> Self {
        ListWidget { show_column_names, ..self }
    }

    pub fn set_highlighted(&mut self, highlighted: usize) -> bool {
        if self.items().count() >= highlighted {
            self.highlighted = Some(highlighted);
            true
        } else {
            false
        }
    }

    pub fn with_size_policy(self, fill_policy: SizePolicy) -> Self {
        Self { fill_policy, ..self }
    }

    pub fn set_fill_policy(&mut self, fill_policy: SizePolicy) {
        self.fill_policy = fill_policy;
    }

    pub fn get_fill_policy(&self) -> SizePolicy {
        self.fill_policy
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

    pub fn get_highlighted_item(&self) -> Option<&Item> {
        self.highlighted.map(|idx| self.provider.items().nth(idx)).flatten()
    }

    pub fn full_size_from_items(&self) -> XY {
        // TODO overflow
        let rows = self.items().count() as u16 + if self.show_column_names { 1 } else { 0 } as u16;
        let mut cols = 0;

        for i in 0..Item::len_columns() {
            cols += Item::get_min_column_width(i);
        }

        XY::new(cols, rows)
    }
}

impl<Item: ListWidgetItem + 'static> Widget for ListWidget<Item> {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        TYPENAME
    }
    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        TYPENAME
    }
    fn full_size(&self) -> XY {
        self.full_size_from_items()
    }

    fn size_policy(&self) -> SizePolicy {
        self.fill_policy
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.last_size = Some(screenspace);
    }

    fn kite(&self) -> XY {
        // TODO overflow
        XY::new(0, self.highlighted.unwrap_or(0) as u16)
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            InputEvent::KeyInput(key) => match key.keycode {
                Keycode::ArrowUp => {
                    if self.highlighted.map(|f| f > 0).unwrap_or(false) {
                        Some(ListWidgetMsg::Arrow(Arrow::Up))
                    } else {
                        None
                    }
                }
                Keycode::ArrowDown => {
                    if self.highlighted.map(|f| f + 1 < self.provider.items().count()).unwrap_or(false) {
                        Some(ListWidgetMsg::Arrow(Arrow::Down))
                    } else {
                        None
                    }
                }
                Keycode::ArrowLeft => None,
                Keycode::ArrowRight => None,
                Keycode::Enter => {
                    if self.on_hit.is_some() {
                        Some(ListWidgetMsg::Hit)
                    } else {
                        None
                    }
                }
                Keycode::Home => Some(ListWidgetMsg::Home),
                Keycode::End => Some(ListWidgetMsg::End),
                Keycode::PageUp => Some(ListWidgetMsg::PageUp),
                Keycode::PageDown => Some(ListWidgetMsg::PageDown),
                _ => None,
            },
            _ => None,
        }
        .map(|m| Box::new(m) as Box<dyn AnyMsg>);
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        let our_msg = msg.as_msg::<ListWidgetMsg>();
        if our_msg.is_none() {
            warn!("expecetd ListWidgetMsg, got {:?}", msg);
            return None;
        }

        return match our_msg.unwrap() {
            ListWidgetMsg::Arrow(arrow) => match self.highlighted {
                None => None,
                Some(old_highlighted) => match arrow {
                    Arrow::Up => {
                        if old_highlighted > 0 {
                            self.highlighted = Some(old_highlighted - 1);
                            self.on_change()
                        } else {
                            self.on_miss()
                        }
                    }
                    Arrow::Down => {
                        let provider_len = self.provider.items().count();
                        debug!("items {}, old_high {}", provider_len, old_highlighted);
                        if old_highlighted + 1 < provider_len {
                            self.highlighted = Some(old_highlighted + 1);
                            self.on_change()
                        } else {
                            self.on_miss()
                        }
                    }
                    Arrow::Left => None,
                    Arrow::Right => None,
                },
            },
            ListWidgetMsg::Hit => self.on_hit(),
            ListWidgetMsg::Home => None,
            ListWidgetMsg::End => None,
            ListWidgetMsg::PageUp => {
                if let Some(highlighted) = self.highlighted {
                    if highlighted > 0 {
                        let last_size = unpack_or_e!(self.last_size, None, "page_up before layout");
                        let page_height = last_size.page_height();
                        let preferred_idx = if highlighted > page_height as usize {
                            highlighted - page_height as usize
                        } else {
                            0
                        };

                        let count = self.items().take(preferred_idx + 1).count();
                        if count > preferred_idx {
                            self.highlighted = Some(preferred_idx);
                        } else {
                            if count > 0 {
                                self.highlighted = Some(count - 1);
                            } else {
                                self.highlighted = None;
                            }
                        }
                        self.on_change()
                    } else {
                        self.on_miss()
                    }
                } else {
                    self.on_miss()
                }
            }
            ListWidgetMsg::PageDown => {
                if let Some(highlighted) = self.highlighted {
                    let last_size = unpack_or_e!(self.last_size, None, "page_down before layout");
                    let page_height = last_size.page_height();
                    let preferred_idx = highlighted + page_height as usize;

                    let count = self.items().take(preferred_idx + 1).count();
                    if count > 0 {
                        self.highlighted = Some(count - 1);
                        self.on_change()
                    } else {
                        error!("something was highlighted, now provider is empty. That's an error described in focus_and_input.md in section about subscriptions");
                        self.on_miss()
                    }
                } else {
                    self.on_miss()
                }
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let _size = unpack_or_e!(self.last_size, (), "render before layout");
        #[cfg(test)]
        output.emit_metadata(crate::io::output::Metadata {
            id: self.id(),
            typename: self.typename().to_string(),
            rect: crate::primitives::rect::Rect::from_zero(_size.output_size()),
            focused,
        });

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

        for (line_idx, item) in self.items().enumerate() {
            debug!(
                "y+idx = {}, osy = {:?}, item = {:?}",
                y_offset as usize + line_idx,
                output.size().y,
                item
            );

            if y_offset as usize + line_idx >= size.output_size().y as usize {
                break;
            }

            let mut x_offset: u16 = 0;

            let style = if self.highlighted == Some(line_idx) {
                cursor_style
            } else {
                primary_style
            };

            for column_idx in 0..Item::len_columns() {
                if x_offset >= size.output_size().x {
                    warn!(
                        "completely skipping drawing a column {} and consecutive, because it's offset is beyond output",
                        column_idx
                    );
                    break;
                }

                // it get's at most "what's left", but not more than it requires
                let mut column_width = min(Item::get_min_column_width(column_idx), size.output_size().x - x_offset);
                debug_assert!(column_width > 0);

                // that is unless it's a last cell, then it gets all there is
                if column_idx + 1 == Item::len_columns() {
                    column_width = size.output_size().x - x_offset;
                }

                let actual_max_text_length = column_width;
                debug_assert!(actual_max_text_length > 0);

                // huge block to "cut th text if necessary"
                let text: String = {
                    let full_text: String = match item.get(column_idx) {
                        Some(s) => s.to_string(),
                        None => "".to_string(),
                    };
                    if full_text.len() <= actual_max_text_length as usize {
                        full_text
                    } else {
                        copy_first_n_columns(&full_text, actual_max_text_length as usize, true).unwrap()
                    }
                };
                debug_assert!(text.width() <= actual_max_text_length as usize);

                match self.query.as_ref().map(|q| q.matches_highlights(&text)) {
                    None => {
                        output.print_at(
                            // TODO possible u16 overflow
                            XY::new(x_offset, y_offset + line_idx as u16),
                            style,
                            &text,
                        );
                    }
                    Some(highlight_iter) => {
                        let mut x_stride: usize = 0;
                        let mut highlight_iter = highlight_iter.peekable();
                        for (grapheme_idx, grapheme) in text.graphemes(true).enumerate() {
                            let highlight = highlight_iter.peek() == Some(&grapheme_idx);
                            if highlight {
                                highlight_iter.next();
                            }

                            let style = if !highlight { style } else { theme.highlighted(focused) };

                            output.print_at(XY::new(x_offset + (x_stride as u16), y_offset + line_idx as u16), style, grapheme);

                            x_stride += grapheme.width();
                        }
                    }
                }

                if text.width() < column_width as usize {
                    // since text.width() is < column_width, it's safe to cast to u16.
                    for x_stride in (text.width() as u16)..column_width {
                        let pos = XY::new(x_offset + x_stride, y_offset + line_idx as u16);
                        // debug!("printing at pos {} size {}", pos, output.size());
                        output.print_at(
                            // TODO possible u16 oveflow
                            pos, style, " ",
                        );
                    }
                }

                x_offset += Item::get_min_column_width(column_idx);
            }
        }
    }
}
