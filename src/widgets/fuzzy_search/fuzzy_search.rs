use log::{debug, warn};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{AnyMsg, InputEvent, Keycode, Output, SizeConstraint, Widget, ZERO};
use crate::io::sub_output::SubOutput;
use crate::primitives::cursor_set::CursorStatus;
use crate::primitives::rect::Rect;
use crate::primitives::theme::Theme;
use crate::primitives::xy::XY;
use crate::widget::widget::{get_new_widget_id, WID, WidgetAction};
use crate::widgets::common_edit_msgs::key_to_edit_msg;
use crate::widgets::edit_box::{EditBoxWidget, EditBoxWidgetMsg};
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};
use crate::widgets::fuzzy_search::msg::{FuzzySearchMsg, Navigation};

pub struct FuzzySearchWidget {
    id: WID,
    edit: EditBoxWidget,
    providers: Vec<Box<dyn ItemsProvider>>,
    context_shortcuts: Vec<String>,

    highlighted: usize,

    on_close: WidgetAction<Self>,
    on_miss: Option<WidgetAction<Self>>,
}

impl FuzzySearchWidget {
    pub fn new(on_close: WidgetAction<Self>) -> Self {
        let edit = EditBoxWidget::new();

        Self {
            id: get_new_widget_id(),
            edit,
            providers: vec![],
            context_shortcuts: vec![],
            highlighted: 0,
            on_close,
            on_miss: None,
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
        16
    }

    fn items(&self) -> ItemIter {
        ItemIter {
            providers: &self.providers,
            context_shortcuts: &self.shortened_contexts(),
            query: self.edit.get_text().to_string(),
            pos: 0,
            cur_iter: None,
        }
    }
}

struct ItemIter<'a> {
    providers: &'a Vec<Box<dyn ItemsProvider>>,
    context_shortcuts: &'a Vec<String>,
    query: String,
    pos: usize,
    cur_iter: Option<Box<dyn Iterator<Item=&'a dyn Item> + 'a>>,
}

impl<'a> Iterator for ItemIter<'a> {
    type Item = &'a dyn Item;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.providers.len() {
            if self.cur_iter.is_none() {
                self.cur_iter = Some(self.providers[self.pos].items(self.query.clone()));
            }

            let iter = self.cur_iter.as_mut().unwrap();

            match iter.next() {
                Some(item) => { return Some(item); }
                None => {
                    self.cur_iter = None;
                    self.pos += 1;
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

    fn layout(&mut self, _sc: SizeConstraint) -> XY {
        let items_len = self.items().count() + 1;

        //TODO
        XY::new(16, items_len as u16)
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
                        if self.highlighted < self.items().count() + 1 {
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

        //

        let query = self.edit.get_text().to_string();

        for (item_idx, item) in self.items().enumerate() {
            let mut query_it = query.graphemes(true).peekable();
            let mut x = 0 as u16;

            let selected_line = item_idx == self.highlighted;

            for g in item.display_name().graphemes(true) {
                let selected_grapheme = query_it.peek().map(|f| *f == g).unwrap_or(false);

                let mut style = if selected_line {
                    theme.highlighted(focused)
                } else {
                    theme.default_text(focused)
                };

                if selected_grapheme {
                    theme.cursor_background(CursorStatus::WithinSelection).map(|bg| {
                        style = style.with_background(bg);
                    });
                }

                output.print_at(
                    XY::new(x, item_idx as u16 + 1),
                    style,
                    g,
                );

                x += g.width_cjk() as u16;
                if selected_grapheme {
                    query_it.next();
                }
            }
        }
    }
}