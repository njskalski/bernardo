use std::cmp::min;
use log::{debug, error, warn};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{AnyMsg, InputEvent, Keycode, Output, SizeConstraint, Widget, ZERO};
use crate::io::sub_output::SubOutput;
use crate::primitives::cursor_set::CursorStatus;
use crate::primitives::rect::Rect;
use crate::primitives::theme::Theme;
use crate::primitives::xy::XY;
use crate::widget::widget::{get_new_widget_id, WID, WidgetAction};
use crate::primitives::common_edit_msgs::key_to_edit_msg;
use crate::widgets::edit_box::{EditBoxWidget, EditBoxWidgetMsg};
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};
use crate::widgets::fuzzy_search::msg::{FuzzySearchMsg, Navigation};

const DEFAULT_WIDTH: u16 = 16;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DrawComment {
    None,
    Highlighted,
    All,
}

pub struct FuzzySearchWidget {
    id: WID,
    edit: EditBoxWidget,
    providers: Vec<Box<dyn ItemsProvider>>,
    context_shortcuts: Vec<String>,

    draw_comment: DrawComment,

    last_height_limit: Option<u16>,

    highlighted: usize,

    on_close: WidgetAction<Self>,
    on_miss: Option<WidgetAction<Self>>,
    // on_hit is a part of PROVIDER.

    width: u16,
}

impl FuzzySearchWidget {
    pub fn new(on_close: WidgetAction<Self>) -> Self {
        let edit = EditBoxWidget::new();

        Self {
            id: get_new_widget_id(),
            edit,
            providers: Vec::default(),
            context_shortcuts: Vec::default(),
            draw_comment: DrawComment::None,
            last_height_limit: None,
            highlighted: 0,
            on_close,
            on_miss: None,
            width: DEFAULT_WIDTH,
        }
    }

    pub fn with_provider(self, provider: Box<dyn ItemsProvider>) -> Self {
        let mut contexts = self.providers;
        contexts.push(provider);

        let mut context_shortcuts: Vec<String> = vec![];
        context_shortcuts.resize(contexts.len(), String::default());

        // TODO add common prefixes bla bla bla
        for (idx, c) in contexts.iter().enumerate() {
            context_shortcuts[idx] += c.context_name().graphemes(true).next().unwrap_or("");
        }

        Self {
            providers: contexts,
            context_shortcuts,
            ..self
        }
    }

    pub fn with_draw_comment_setting(self, draw_context_setting: DrawComment) -> Self {
        Self {
            draw_comment: draw_context_setting,
            ..self
        }
    }

    pub fn set_draw_comment_setting(&mut self, draw_context_setting: DrawComment) {
        self.draw_comment = draw_context_setting;
    }

    pub fn on_miss(self, on_miss: WidgetAction<Self>) -> Self {
        Self {
            on_miss: Some(on_miss),
            ..self
        }
    }

    pub fn shortened_contexts(&self) -> &Vec<String> {
        &self.context_shortcuts
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn items(&self) -> ItemIter {
        let rows_limit = self.last_height_limit.unwrap_or_else(|| {
            error!("items called before last_height_limit set, using 128 as 'safe default'");
            128
        });

        ItemIter {
            providers: &self.providers,
            context_shortcuts: &self.shortened_contexts(),
            query: self.edit.get_text().to_string(),
            rows_limit: rows_limit as usize,
            provider_idx: 0,
            cur_iter: None,
        }
    }
}

struct ItemIter<'a> {
    providers: &'a Vec<Box<dyn ItemsProvider>>,
    context_shortcuts: &'a Vec<String>,
    query: String,
    rows_limit: usize,
    provider_idx: usize,
    cur_iter: Option<Box<dyn Iterator<Item=Box<dyn Item + 'a>> + 'a>>,
}


impl<'a> Iterator for ItemIter<'a> {
    type Item = Box<dyn Item + 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rows_limit == 0 {
            return None;
        }

        while self.provider_idx < self.providers.len() {
            if self.cur_iter.is_none() {
                self.cur_iter = Some(self.providers[self.provider_idx].items(self.query.clone(), self.rows_limit));
            }

            let iter = self.cur_iter.as_mut().unwrap();

            match iter.next() {
                Some(item) => {
                    self.rows_limit -= 1;
                    return Some(item);
                }
                None => {
                    self.cur_iter = None;
                    self.provider_idx += 1;
                }
            }
        }

        None
    }
}

impl Widget for FuzzySearchWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn typename(&self) -> &'static str {
        "fuzzy_search"
    }

    fn min_size(&self) -> XY {
        // Completely arbitrary
        XY::new(16, 5)
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {

        // This is a reasonable assumption: I never want to display more elements in fuzzy search that
        // can be displayed on a "physical" screen. Even if fuzzy is inside a scroll, the latest position
        // I might be interested in is "lower_right().y".
        self.last_height_limit = Some(sc.visible_hint().lower_right().y);

        self.edit.layout(sc);
        self.width = sc.visible_hint().size.x;

        let items_len = match self.draw_comment {
            DrawComment::None => self.items().count() + 1,
            DrawComment::Highlighted => self.items().count() + 2,
            DrawComment::All => self.items().count() * 2 + 1,
        };

        //TODO
        XY::new(self.width, min(items_len as u16, sc.visible_hint().lower_right().y))
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            InputEvent::KeyInput(ki) => {
                if ki.keycode == Keycode::Esc {
                    return Some(Box::new(FuzzySearchMsg::Close));
                }

                if ki.keycode == Keycode::Enter {
                    return Some(Box::new(FuzzySearchMsg::Hit));
                }

                let nav_msg = match ki.keycode {
                    Keycode::ArrowUp => Some(FuzzySearchMsg::Navigation(Navigation::ArrowUp)),
                    Keycode::ArrowDown => Some(FuzzySearchMsg::Navigation(Navigation::ArrowDown)),
                    Keycode::PageUp => Some(FuzzySearchMsg::Navigation(Navigation::PageUp)),
                    Keycode::PageDown => Some(FuzzySearchMsg::Navigation(Navigation::PageDown)),
                    _ => None,
                };

                match nav_msg {
                    Some(msg) => Some(Box::new(msg)),
                    None => {
                        match key_to_edit_msg(ki) {
                            Some(cem) => Some(Box::new(FuzzySearchMsg::EditMsg(cem))),
                            None => None,
                        }
                    }
                }
            }
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("fuzzy_search.update {:?}", msg);

        let our_msg = msg.as_msg::<FuzzySearchMsg>();
        if our_msg.is_none() {
            warn!("expecetd FuzzySearchMsg, got {:?}", msg);
            return None;
        }

        match our_msg.unwrap() {
            FuzzySearchMsg::EditMsg(cem) => self.edit.update(Box::new(EditBoxWidgetMsg::CommonEditMsg(cem.clone()))),
            FuzzySearchMsg::EscalateContext => None, //TODO
            FuzzySearchMsg::Navigation(nav) => {
                match nav {
                    Navigation::PageUp => {}
                    Navigation::PageDown => {}
                    Navigation::ArrowUp => {
                        if self.highlighted > 0 {
                            self.highlighted -= 1;
                        }
                    }
                    Navigation::ArrowDown => {
                        if self.highlighted + 1 < self.items().count() {
                            self.highlighted += 1;
                        }
                    }
                }
                None //TODO
            }
            FuzzySearchMsg::Close => {
                (self.on_close)(self)
            }
            FuzzySearchMsg::Hit => {
                match self.items().nth(self.highlighted) {
                    Some(item) => Some(item.on_hit()),
                    None => None
                }
            }
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let mut suboutput = SubOutput::new(output,
                                           Rect::new(ZERO, XY::new(self.width(), 1)));

        self.edit.render(theme, focused, &mut suboutput);
        let query = self.edit.get_text().to_string();

        let mut y = 1 as u16;

        for (item_idx, item) in self.items().enumerate() {
            let mut query_it = query.graphemes(true).peekable();
            let mut x = 0 as u16;

            let selected_line = item_idx == self.highlighted;

            let style = if selected_line {
                theme.highlighted(focused)
            } else {
                theme.default_text(focused)
            };

            for g in item.display_name().graphemes(true) {
                let selected_grapheme = query_it.peek().map(|f| *f == g).unwrap_or(false);
                let grapheme_style = if selected_grapheme {
                    theme.cursor_background(CursorStatus::WithinSelection).map(|bg| {
                        style.with_background(bg)
                    }).unwrap_or(style)
                } else { style };

                output.print_at(
                    XY::new(x, y),
                    grapheme_style,
                    g,
                );

                x += g.width_cjk() as u16;
                if selected_grapheme {
                    query_it.next();
                }
            }

            //TODO cast overflow
            for x in (item.display_name().width_cjk() as u16)..self.width() {
                output.print_at(XY::new(x as u16, y),
                                style,
                                " ");
            }

            if self.draw_comment == DrawComment::All || (self.draw_comment == DrawComment::Highlighted && selected_line) {
                if let Some(comment) = item.comment() {
                    let mut x = 0 as u16;
                    for g in comment.graphemes(true) {
                        output.print_at(XY::new(x, y + 1),
                                        style,
                                        g);
                        x += g.width_cjk() as u16; //TODO overflow
                    }
                    //TODO cast overflow
                    for x in (comment.width_cjk() as u16)..self.width() {
                        output.print_at(XY::new(x as u16, y + 1),
                                        style,
                                        " ");
                    }
                }
            }

            y += match self.draw_comment {
                DrawComment::None => 1,
                DrawComment::Highlighted => if selected_line && item.comment().is_some() { 2 } else { 1 },
                DrawComment::All => if item.comment().is_some() { 2 } else { 1 },
            };
        }
    }
}