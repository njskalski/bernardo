use std::cmp::min;

use log::{debug, error, warn};
use streaming_iterator::StreamingIterator;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{AnyMsg, InputEvent, Keycode, Output, SizeConstraint, Widget, ZERO};
use crate::config::theme::Theme;
use crate::io::sub_output::SubOutput;
use crate::primitives::common_edit_msgs::key_to_edit_msg;
use crate::primitives::cursor_set::CursorStatus;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::{get_new_widget_id, WID, WidgetAction};
use crate::widgets::edit_box::{EditBoxWidget, EditBoxWidgetMsg};
use crate::widgets::fuzzy_search::item_provider::{FuzzyItem, FuzzyItemsProvider};
use crate::widgets::fuzzy_search::msg::{FuzzySearchMsg, Navigation};

const DEFAULT_WIDTH: u16 = 16;

// TODO fix width calculation - I think it was done BEFORE introduced SizeConstraint

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DrawComment {
    None,
    Highlighted,
    All,
}

pub struct FuzzySearchWidget {
    id: WID,
    edit: EditBoxWidget,
    provider: Box<dyn FuzzyItemsProvider>,

    draw_comment: DrawComment,

    last_height_limit: Option<u16>,

    highlighted: usize,

    on_close: WidgetAction<Self>,
    on_miss: Option<WidgetAction<Self>>,
    // on_hit is a part of PROVIDER.

    width: u16,
}

impl FuzzySearchWidget {
    pub fn new(on_close: WidgetAction<Self>, provider: Box<dyn FuzzyItemsProvider>) -> Self {
        let edit = EditBoxWidget::new();

        Self {
            id: get_new_widget_id(),
            edit,
            provider,
            draw_comment: DrawComment::None,
            last_height_limit: None,
            highlighted: 0,
            on_close,
            on_miss: None,
            width: DEFAULT_WIDTH,
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

    fn width(&self) -> u16 {
        self.width
    }

    fn items(&self) -> Box<dyn StreamingIterator<Item=Box<dyn FuzzyItem + '_>> + '_> {
        self.provider.items(self.edit.get_text().to_string())
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

        let count = self.items().as_mut().count();

        let items_len = match self.draw_comment {
            DrawComment::None => count + 1,
            DrawComment::Highlighted => count + 2,
            DrawComment::All => count * 2 + 1,
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
                        if self.highlighted + 1 < self.items().as_mut().count() {
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
        let mut items = self.items();
        let mut item_idx = 0 as usize;

        while let Some(item) = items.next() {
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

                if output.size_constraint().x().map(|max_x| x >= max_x).unwrap_or(false) {
                    break;
                }
            }

            //TODO cast overflow
            for x in (item.display_name().as_ref().width_cjk() as u16)..self.width() {
                output.print_at(XY::new(x as u16, y),
                                style,
                                " ");
            }

            if self.draw_comment == DrawComment::All || (self.draw_comment == DrawComment::Highlighted && selected_line) {
                if let Some(comment) = item.comment() {
                    let mut x = 0 as u16;
                    for g in comment.as_ref().graphemes(true) {
                        output.print_at(XY::new(x, y + 1),
                                        style,
                                        g);
                        x += g.width_cjk() as u16; //TODO overflow
                    }
                    //TODO cast overflow
                    for x in (comment.as_ref().width_cjk() as u16)..self.width() {
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

            if y >= output.size_constraint().visible_hint().lower_right().y {
                break;
            }

            item_idx += 1;
        }
    }
}