use std::cmp::max;

use log::{debug, error, warn};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::cursor::cursor::CursorStatus;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::primitives::common_edit_msgs::key_to_edit_msg;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::unpack_unit_e;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::{get_new_widget_id, Widget, WidgetAction, WID};
use crate::widgets::edit_box::{EditBoxWidget, EditBoxWidgetMsg};
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};
use crate::widgets::fuzzy_search::msg::{FuzzySearchMsg, Navigation};

/* TODO I am not sure if I want to keep this widget, or do I integrate it with context menu widget now brewing \
slowly somewhere in editor */

// const DEFAULT_WIDTH: u16 = 16;

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

    highlighted: usize,

    on_close: WidgetAction<Self>,
    on_miss: Option<WidgetAction<Self>>,
    // on_hit is a part of PROVIDER.

    // always greedy on X
    last_size: Option<Screenspace>,
}

impl FuzzySearchWidget {
    pub const TYPENAME: &'static str = "fuzzy_search";

    pub fn new(on_close: WidgetAction<Self>, clipboard_op: Option<ClipboardRef>) -> Self {
        let mut edit = EditBoxWidget::new();

        match clipboard_op {
            None => {}
            Some(clipboard) => {
                edit = edit.with_clipboard(clipboard);
            }
        }

        Self {
            id: get_new_widget_id(),
            edit,
            providers: Vec::default(),
            context_shortcuts: Vec::default(),
            draw_comment: DrawComment::None,
            highlighted: 0,
            on_close,
            on_miss: None,
            last_size: None,
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

    fn items(&self) -> ItemIter {
        let rows_limit = self.last_size.map(|ss| ss.output_size().y).unwrap_or_else(|| {
            error!("items called before last_size set, using 128 as 'safe default'");
            128
        });

        ItemIter {
            providers: &self.providers,
            context_shortcuts: &self.shortened_contexts(),
            query: self.edit.get_buffer().to_string(),
            rows_limit: rows_limit as usize,
            provider_idx: 0,
            cur_iter: None,
        }
    }

    fn size_from_items(&self) -> XY {
        let mut res = XY::ONE;

        for (idx, item) in self.items().enumerate() {
            // TODO overflow

            let y = match self.draw_comment {
                DrawComment::None => 1,
                DrawComment::Highlighted => {
                    if self.highlighted == idx {
                        2
                    } else {
                        1
                    }
                }
                DrawComment::All => 2,
            };
            let mut local_xy = XY::new(item.display_name().width() as u16, y);

            if idx == self.highlighted {
                local_xy.x = max(item.comment().map(|c| c.width() as u16).unwrap_or(0), local_xy.x);
            }

            res.x = max(res.x, local_xy.x);
            res.y += local_xy.y;
        }

        let edit_min = self.edit.full_size();
        res.x = max(res.x, edit_min.x);
        res.y += edit_min.y;

        res
    }
}

struct ItemIter<'a> {
    providers: &'a Vec<Box<dyn ItemsProvider>>,
    context_shortcuts: &'a Vec<String>,
    query: String,
    rows_limit: usize,
    provider_idx: usize,
    cur_iter: Option<Box<dyn Iterator<Item = Box<dyn Item + 'a>> + 'a>>,
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
        Self::TYPENAME
    }
    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }
    fn full_size(&self) -> XY {
        self.size_from_items()
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.last_size = Some(screenspace);
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
                    None => match key_to_edit_msg(ki) {
                        Some(cem) => Some(Box::new(FuzzySearchMsg::EditMsg(cem))),
                        None => None,
                    },
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
            FuzzySearchMsg::Close => (self.on_close)(self),
            FuzzySearchMsg::Hit => match self.items().nth(self.highlighted) {
                Some(item) => Some(item.on_hit()),
                None => None,
            },
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let size = unpack_unit_e!(self.last_size, "render before layout",);

        #[cfg(test)]
        {
            output.emit_metadata(crate::io::output::Metadata {
                id: self.id,
                typename: self.typename().to_string(),
                rect: Rect::from_zero(output.size()),
                focused,
            });
        }

        let mut suboutput = SubOutput::new(output, Rect::new(XY::ZERO, XY::new(size.output_size().x, 1)));

        self.edit.render(theme, focused, &mut suboutput);
        let query = self.edit.get_buffer().to_string();

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

            for g in item.display_name().as_ref().graphemes(true) {
                let selected_grapheme = query_it.peek().map(|f| *f == g).unwrap_or(false);
                let grapheme_style = if selected_grapheme {
                    theme
                        .cursor_background(CursorStatus::WithinSelection)
                        .map(|bg| style.with_background(bg))
                        .unwrap_or(style)
                } else {
                    style
                };

                output.print_at(XY::new(x, y), grapheme_style, g);

                x += g.width() as u16;
                if selected_grapheme {
                    query_it.next();
                }

                if x > output.size().x {
                    break;
                }
            }

            //TODO cast overflow
            for x in (item.display_name().as_ref().width() as u16)..size.output_size().x {
                output.print_at(XY::new(x as u16, y), style, " ");
            }

            if self.draw_comment == DrawComment::All || (self.draw_comment == DrawComment::Highlighted && selected_line) {
                if let Some(comment) = item.comment() {
                    let mut x = 0 as u16;
                    for g in comment.as_ref().graphemes(true) {
                        output.print_at(XY::new(x, y + 1), style, g);
                        x += g.width() as u16; //TODO overflow
                    }
                    //TODO cast overflow
                    for x in (comment.as_ref().width() as u16)..size.output_size().x {
                        output.print_at(XY::new(x as u16, y + 1), style, " ");
                    }
                }
            }

            y += match self.draw_comment {
                DrawComment::None => 1,
                DrawComment::Highlighted => {
                    if selected_line && item.comment().is_some() {
                        2
                    } else {
                        1
                    }
                }
                DrawComment::All => {
                    if item.comment().is_some() {
                        2
                    } else {
                        1
                    }
                }
            };

            if y >= size.output_size().y {
                break;
            }
        }
    }

    fn kite(&self) -> XY {
        //TODO overflow
        XY::new(self.highlighted as u16, 0)
    }
}
