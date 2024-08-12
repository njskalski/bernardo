use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::time::Duration;

use log::{debug, error, warn};
use matches::debug_assert_matches;
use streaming_iterator::StreamingIterator;
use unicode_width::UnicodeWidthStr;

use crate::config::theme::Theme;
use crate::cursor::cursor::{Cursor, CursorStatus, Selection};
use crate::cursor::cursor_set::CursorSet;
use crate::cursor::cursor_set_rect::cursor_set_to_rect;
use crate::experiments::regex_search::FindError;
use crate::experiments::screenspace::Screenspace;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::Output;
use crate::io::style::TextStyle;
use crate::io::sub_output::SubOutput;
use crate::primitives::arrow::Arrow;
use crate::primitives::color::Color;
use crate::primitives::common_edit_msgs::{apply_common_edit_message, cme_to_direction, key_to_edit_msg, CommonEditMsg};
use crate::primitives::has_invariant::HasInvariant;
use crate::primitives::helpers;
use crate::primitives::printable::Printable;
use crate::primitives::rect::Rect;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::primitives::styled_printable::StyledPrintable;
use crate::primitives::xy::XY;
use crate::promise::promise::PromiseState;
use crate::text::buffer_state::BufferState;
use crate::text::text_buffer::TextBuffer;
use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
use crate::w7e::handler::NavCompRef;
use crate::w7e::navcomp_provider::CompletionAction;
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};
use crate::widgets::code_results_view::promise_provider::WrappedSymbolUsagesPromise;
use crate::widgets::editor_widget::completion::completion_widget::CompletionWidget;
use crate::widgets::editor_widget::context_bar::widget::ContextBarWidget;
use crate::widgets::editor_widget::context_options_matrix::get_context_options;
use crate::widgets::editor_widget::helpers::{find_trigger_and_substring, CursorScreenPosition};
use crate::widgets::editor_widget::label::label::Label;
use crate::widgets::editor_widget::msg::EditorWidgetMsg;
use crate::widgets::main_view::msg::MainViewMsg;
use crate::{unpack_or, unpack_or_e, unpack_unit, unpack_unit_e};

const MIN_EDITOR_SIZE: XY = XY::new(10, 3);
// const MAX_HOVER_SIZE: XY = XY::new(64, 20);

const NEWLINE: &str = "⏎";
const BEYOND: &str = "⇱";

const DEFAULT_EDITOR_TIMEOUT: Duration = Duration::from_millis(500);

/*
This is heart and soul of Gladius Editor.

One important thing is that unless I come up with a great idea about scrolling, the "OverOutput"
has to be last output passed to render function, otherwise scrolling will fail to work.

I quite often use "screen position" or "screen space", but I actually mean "widget space", because
each widget is given their own "output" to render at.
 */

/*
TODO:
- display save error (and write tests)
- support for files with more than u16 lines
- I should probably remember "milestones" every several mb of file for big files.
- we have a lot of places where we check for Some on path, single cursor, navcomp and cast from cursor to stupid cursor.
    they need unification.
- backspace beyond trigger should close completions
- completions should be anchored to trigger, not to cursor
- I don't think the the layout() follows the invariant I set in widget
 */

/*
Yes, I know it should be private, but to be honest an opportunity to break apart a potentially
several thousand lines long file into logical components was more important to me than visibility
rules.
*/
#[derive(Debug)]
pub enum EditorState {
    Editing,
    /*
    Dropping cursor mode works in a following way:
    we have a special cursor (different color) and background is different (to tell apart "mode").
    we move this special cursor with arrows and whenever hit ENTER a new cursor is created/removed
    under the position of special cursor.

    moving special cursor with ctrl (jumping words) and code navigation shall work normally.
    furthermore, typing leads to search-and-highlight, as opposed to editing events.

    Due to limited time resources, at this moment I will not solve an issue of dropping a cursor
    more than one character beyond the buffer.
     */
    DroppingCursor { special_cursor: Cursor },
}

/*
These settings are an aggregate of "everything layout will need to position and layout a hover"
 */
#[derive(Debug)]
pub struct HoverSettings {
    /*
    anchor is the character *in the line* (so it's never included in the rect of hover).
    And it is:
    - position of last character of trigger (if available)
    - just position of cursor

    Anchor is given in widget space.
    */
    pub anchor: XY,
    pub cursor_screen_position: CursorScreenPosition,
    pub substring: Option<String>,
    pub trigger: Option<String>,
}

enum EditorHover {
    Completion(CompletionWidget),
    Context(ContextBarWidget),
}

impl EditorHover {
    fn get_widget(&self) -> &dyn Widget {
        match self {
            EditorHover::Completion(cw) => cw,
            EditorHover::Context(cw) => cw,
        }
    }

    fn get_widget_mut(&mut self) -> &mut dyn Widget {
        match self {
            EditorHover::Completion(cw) => cw,
            EditorHover::Context(cw) => cw,
        }
    }
}

pub struct EditorWidget {
    wid: WID,
    providers: Providers,

    readonly: bool,
    // I'd prefer to have "constrain cursors to visible part", but since it's non-trivial, I don't do it now.
    ignore_input_altogether: bool,

    layout_res: Option<Screenspace>,

    // to be constructed in layout step based on HoverSettings
    last_hover_rect: Option<Rect>,

    buffer: BufferSharedRef,

    kite: XY,

    // navcomp is to submit edit messages, suggestion display will probably be somewhere else
    navcomp: Option<NavCompRef>,

    // TODO this was supposed to be a "symbol under cursor" but of course it doesn't work this way.
    // navcomp_symbol: Option<Box<dyn Promise<Option<NavCompSymbol>>>>,
    state: EditorState,

    // This is completion or navigation
    // Settings are calculated based on last_size, and entire hover will be discarded on resize.
    requested_hover: Option<(HoverSettings, EditorHover)>,
    // These are label providers. Their order is important.
    // todo_lable_providers: Vec<LabelsProviderRef>, // moved to providers
}

impl EditorWidget {
    pub const TYPENAME: &'static str = "editor_widget";

    const MAX_HOVER_WIDTH: u16 = 45;
    const MIN_HOVER_WIDTH: u16 = 15;

    pub fn new(providers: Providers, buffer: BufferSharedRef) -> EditorWidget {
        let buffer_named: bool = buffer.lock().map(|lock| lock.get_path().is_some()).unwrap_or_else(|| {
            error!("failed to lock buffer");
            false
        });

        let mut res = EditorWidget {
            wid: get_new_widget_id(),
            providers,
            readonly: false,
            ignore_input_altogether: false,
            layout_res: None,
            last_hover_rect: None,
            buffer: buffer.clone(),
            kite: XY::ZERO,
            state: EditorState::Editing,
            navcomp: None,
            requested_hover: None,
        };

        if buffer_named {
            res.after_path_change();
        }

        match buffer.lock_rw() {
            Some(mut buffer_lock) => {
                debug_assert!(buffer_lock.initialize_for_widget(res.wid, None));
            }
            None => {
                error!("failed to lock buffer for rw, shit will blow up");
            }
        }

        res
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        self.readonly = readonly;
    }

    pub fn with_readonly(self) -> Self {
        Self { readonly: true, ..self }
    }

    pub fn with_ignore_input_altogether(self) -> Self {
        Self {
            ignore_input_altogether: true,
            ..self
        }
    }

    pub fn set_ignore_input_altogether(&mut self, ignore_input_altogether: bool) {
        self.ignore_input_altogether = ignore_input_altogether
    }

    fn after_path_change(&mut self) {
        self.update_navcomp();
    }

    pub fn get_buffer(&self) -> &BufferSharedRef {
        &self.buffer
    }

    fn update_navcomp(&mut self) {
        let buffer = unpack_unit_e!(self.buffer.lock(), "failed locking buffer",);

        if self.navcomp.is_none() {
            let navcomp_group = unpack_unit_e!(self.providers.navcomp_group().try_read().ok(), "failed to lock navcompgroup",);

            let navcomp = if let Some(path) = buffer.get_path() {
                navcomp_group.get_navcomp_for(path)
            } else if let Some(lang) = buffer.get_lang_id() {
                navcomp_group.get_navcomp_for_lang(lang)
            } else {
                None
            };

            self.navcomp = navcomp;
        }

        match (self.navcomp.as_ref(), buffer.get_path()) {
            (Some(navcomp), Some(spath)) => {
                navcomp.file_open_for_edition(spath, buffer.text().rope().clone());
            }
            (Some(_navcomp), None) => {
                warn!("unimplemented variant - set navcomp but not path");
            }
            _ => {
                debug!(
                    "not starting navigation, because navcomp is some: {}, ff is some: {}",
                    self.navcomp.is_some(),
                    buffer.get_path().is_some()
                )
            }
        }
    }

    // This updates the "anchor" of view to match the direction of editing. Remember, the scroll will
    // follow the "anchor" with least possible change.
    fn update_kite(&mut self, buffer: &BufferState, last_move_direction: Arrow) {
        // TODO test
        // TODO cleanup - now cursor_set is part of buffer, we can move cursor_set_to_rect method there

        let cursor_set = unpack_unit_e!(buffer.text().get_cursor_set(self.wid), "failed to get cursor_set",);

        let cursor_rect = cursor_set_to_rect(cursor_set, buffer);
        match last_move_direction {
            Arrow::Up => {
                if self.kite.y > cursor_rect.upper_left().y {
                    self.kite.y = cursor_rect.upper_left().y;
                }
            }
            Arrow::Down => {
                if self.kite.y < cursor_rect.lower_right().y {
                    self.kite.y = cursor_rect.lower_right().y;
                }
            }
            Arrow::Left => {
                if self.kite.x > cursor_rect.upper_left().x {
                    self.kite.x = cursor_rect.upper_left().x;
                }
            }
            Arrow::Right => {
                if self.kite.x < cursor_rect.lower_right().x {
                    self.kite.x = cursor_rect.lower_right().x;
                }
            }
        }
    }

    pub fn enter_dropping_cursor_mode(&mut self, buffer: &BufferState) {
        debug_assert_matches!(self.state, EditorState::Editing);

        // TODO there was something called "supercursor" that seems to be useful here
        let cursors = unpack_unit_e!(buffer.text().get_cursor_set(self.wid), "failed getting cursor set",);
        self.state = EditorState::DroppingCursor {
            special_cursor: cursors.iter().next().copied().unwrap_or_else(|| {
                warn!("empty cursor set!");
                Cursor::single()
            }),
        };
    }

    pub fn enter_editing_mode(&mut self) {
        debug_assert_matches!(self.state, EditorState::DroppingCursor { .. });
        self.state = EditorState::Editing;
    }

    pub fn get_single_cursor_screen_pos(&self, buffer: &BufferState, cursor: Cursor) -> Option<CursorScreenPosition> {
        let lsp_cursor = unpack_or_e!(
            StupidCursor::from_real_cursor(buffer, cursor).ok(),
            None,
            "failed mapping cursor to lsp-cursor"
        );
        let lsp_cursor_xy = unpack_or_e!(lsp_cursor.to_xy(buffer), None, "lsp cursor beyond XY max");

        let layout_res = unpack_or!(
            self.layout_res.as_ref(),
            None,
            "single_cursor_screen_pos called before first layout"
        );
        let visible_rect = layout_res.visible_rect();

        if !visible_rect.contains(lsp_cursor_xy) {
            warn!("cursor seems to be outside visible hint {:?}", layout_res.visible_rect());
            return Some(CursorScreenPosition {
                cursor,
                widget_space: None,
                text_space: lsp_cursor_xy,
            });
        }

        let local_pos = lsp_cursor_xy - visible_rect.upper_left();

        debug!("cursor {:?} converted to {:?} positioned at {:?}", cursor, lsp_cursor, local_pos);
        debug_assert!(local_pos >= XY::ZERO);
        debug_assert!(local_pos < visible_rect.size);

        Some(CursorScreenPosition {
            cursor,
            widget_space: Some(local_pos),
            text_space: lsp_cursor_xy,
        })
    }

    /*
    This method composes hover_settings that are later used in layout pass.

    triggers - if none, current cursor will become an "anchor".
               if some, function will stride left and aggregate substring between the cursor and ONE OF
                TRIGGERS (in order of appearance, only within visible space)

    TODO what if the trigger happen outside visible space? Do I draw or not?
     */
    pub fn get_cursor_related_hover_settings(&self, buffer: &BufferState, triggers: Option<&Vec<String>>) -> Option<HoverSettings> {
        // let last_size = unpack_or!(self.last_size, None, "requested hover before layout");
        let cursor: Cursor = unpack_or!(
            buffer.cursors(self.wid).and_then(|co| co.as_single()),
            None,
            "multiple cursors or none, not doing hover"
        );
        let cursor_pos = unpack_or!(
            self.get_single_cursor_screen_pos(buffer, cursor),
            None,
            "can't position hover, no cursor local pos"
        );
        let cursor_screen_pos = unpack_or!(cursor_pos.widget_space, None, "no cursor position in screen space");
        // let buffer_r: BufferR = unpack_or!(self.buffer.lock(), None, "failed to lock buffer");
        // let visible_rect = unpack_or!(last_size.visible_hint(), None, "no visible rect - no hover");

        let trigger_and_substring: Option<(&str, String)> =
            triggers.and_then(|triggers| find_trigger_and_substring(triggers, buffer, &cursor_pos));

        let anchor = trigger_and_substring
            .as_ref()
            .map(|tas| {
                let substr_width = tas.1.width() as u16; //TODO overflow
                if substr_width > cursor_screen_pos.x {
                    debug!("absourd");
                    cursor_screen_pos
                } else {
                    cursor_screen_pos - XY::new(substr_width, 0)
                }
            })
            .unwrap_or(cursor_screen_pos);

        let (substring, trigger) = trigger_and_substring
            .map(|tas| (Some(tas.1), Some(tas.0.to_string())))
            .unwrap_or((None, None));
        Some(HoverSettings {
            anchor,
            cursor_screen_position: cursor_pos,
            substring,
            trigger,
        })
    }

    pub fn todo_request_context_bar(&mut self, buffer: &BufferState) {
        debug!("request_context_bar");

        // need to resolve first
        // if let Some(navcomp_symbol) = self.navcomp_symbol.as_mut() {
        //     navcomp_symbol.update();
        // };

        let cursor_set = unpack_unit!(buffer.cursors(self.wid), "no cursor for wid",).clone();

        let single_cursor = cursor_set.as_single();
        let stupid_cursor_op = single_cursor.and_then(|c| StupidCursor::from_real_cursor(buffer, c).ok());

        // let lsp_symbol_op = self.navcomp_symbol.as_ref().map(|ncsp| {
        //     ncsp.read().map(|c| c.as_ref())
        // }).flatten().flatten();

        let char_range_op = single_cursor.map(|a| match a.s {
            None => a.a..a.a + 1,
            Some(sel) => sel.b..sel.e,
        });

        // The reason I unpack and pack char_range_op, because I'm not interested in "all highlights"
        let tree_sitter_highlight = char_range_op
            .map(|range| buffer.highlight(Some(range)))
            .and_then(|highlight_items| {
                // TODO I assume here that "first" is the smallest, it probably is not true
                // debug!("highlight items: [{:?}]", &highlight_items);
                highlight_items.first().map(|c| (*c).clone())
            })
            .map(|highlight_item| highlight_item.identifier);

        let items = get_context_options(
            &self.state,
            single_cursor,
            &cursor_set,
            stupid_cursor_op,
            None,
            tree_sitter_highlight.as_ref().map(|c| c.as_str()),
        );

        if items.is_none() {
            warn!("ignoring everything bar, no items");
            self.requested_hover = None;
        } else {
            let hover_settings_op = self.get_cursor_related_hover_settings(buffer, None);

            self.requested_hover = hover_settings_op.map(|hs| {
                let context_bar = ContextBarWidget::new(self.providers.clone(), items.unwrap())
                    .autoexpand_if_single_subtree()
                    .with_on_hit(|widget| widget.get_highlighted().1.on_hit())
                    .with_on_miss(|_| EditorWidgetMsg::HoverClose.someboxed());

                let hover = EditorHover::Context(context_bar);
                (hs, hover)
            });
        }
    }

    // TODO add test to reformat
    pub fn reformat(&mut self, buffer: &mut BufferState) -> bool {
        let navcomp = unpack_or!(self.navcomp.as_ref(), false, "can't reformat: navcomp not available");
        let path = unpack_or!(buffer.get_path(), false, "can't reformat: unsaved file");
        let mut promise = unpack_or!(navcomp.todo_reformat(path), false, "can't reformat: no promise for reformat");
        let cursor_set = unpack_or!(buffer.cursors(self.wid), false, "no cursor for wid");

        if !cursor_set.are_simple() {
            warn!("can't format: unimplemented for non-simple cursors, sorry");
            return false;
        }

        if promise.wait(Some(DEFAULT_EDITOR_TIMEOUT)) == PromiseState::Ready {
            // invariant : promise ready => take.is_some()
            let edits = unpack_or!(promise.read().unwrap(), false, "can't reformat: promise empty");

            let page_height = self.page_height();

            let _res = buffer.apply_stupid_substitute_messages(self.wid, edits, page_height as usize);

            // This theoretically could be optimised out, but maybe it's not worth it, it leads to
            // a new category of bugs if statement above turns out to be false, and it rarely is,
            // so it's very very hard to test. So I keep this here for peace of mind.
            self.after_content_changed(buffer);

            //res
            true
        } else {
            warn!("reformat promise broken");
            false
        }
    }

    pub fn get_cell_style(
        theme: &Theme,
        cursor_status: CursorStatus,
        is_dropping_cursor: bool,
        is_special_cursor: bool,
        is_focused: bool,
    ) -> TextStyle {
        let mut style = if is_dropping_cursor {
            // TODO move this line to theme
            theme.default_text(is_focused).with_background(theme.ui.mode_2_background)
        } else {
            theme.default_text(is_focused)
        };

        if cursor_status != CursorStatus::None {
            if !is_dropping_cursor {
                debug_assert!(is_special_cursor == false, "can't be a special cursor in edit mode");

                if let Some(cursor_background) = theme.cursor_background(cursor_status) {
                    style.background = cursor_background
                } else {
                    debug_assert!(false, "cursor background None for non-None CursorStatus = {:?}", cursor_status)
                }
            } else {
                // dropping cursor
                if is_special_cursor {
                    style.background = theme.ui.cursors.primary_anchor_background
                }
            }
        }

        if !is_focused {
            style.background = style.background.half();
        }

        style
    }

    pub fn page_height(&self) -> u16 {
        match &self.layout_res {
            Some(layout_res) => layout_res.visible_rect().size.y,
            None => {
                error!("requested height before layout, using {} as page_height instead", MIN_EDITOR_SIZE.y);
                MIN_EDITOR_SIZE.y
            }
        }
    }

    pub fn set_cursors(&mut self, cursor_set: CursorSet) -> bool {
        if cursor_set.len() == 0 {
            error!("empty cursor set!");
            return false;
        }

        let mut max_idx: usize = 0;
        for c in cursor_set.iter() {
            max_idx = max(max_idx, c.a);
            if let Some(s) = c.s {
                max_idx = max(max_idx, s.e);
            }
        }

        let buffer_arc = self.buffer.clone();
        let mut buffer_mut = unpack_or!(buffer_arc.lock_rw(), false, "failed acquiring lock");

        if max_idx > buffer_mut.len_chars() + 1 {
            warn!("can't set cursor at {} for buffer len {}", max_idx, buffer_mut.len_chars());
            return false;
        }

        buffer_mut.text_mut().set_cursor_set(self.wid, cursor_set);
        self.update_kite(&buffer_mut, Arrow::Down);

        true
    }

    pub fn find_once(&mut self, buffer_mut: &mut BufferState, phrase: &str) -> Result<bool, FindError> {
        let res = buffer_mut.find_once(self.wid, phrase);
        if res == Ok(true) {
            // TODO handle "restart from the top"
            self.update_kite(buffer_mut, Arrow::Down);
        }
        res
    }

    fn get_hover_subwidget(&self) -> Option<SubwidgetPointer<Self>> {
        self.requested_hover.as_ref()?;

        Some(SubwidgetPointer::<Self>::new(
            Box::new(|s: &EditorWidget| match s.requested_hover.as_ref().unwrap() {
                (_, EditorHover::Completion(comp)) => comp as &dyn Widget,
                (_, EditorHover::Context(cont)) => cont as &dyn Widget,
            }),
            Box::new(|s: &mut EditorWidget| match s.requested_hover.as_mut().unwrap() {
                (_, EditorHover::Completion(comp)) => comp as &mut dyn Widget,
                (_, EditorHover::Context(cont)) => cont as &mut dyn Widget,
            }),
        ))
    }

    fn can_add_label(labels: &mut BTreeMap<XY, &Label>, new_label: (XY, &Label)) -> bool {
        let width = new_label.1.screen_width();
        let new_rect = Rect::new(new_label.0, XY::new(width, 1));

        for (pos, label) in labels.iter() {
            let width = label.screen_width();
            let old_rect = Rect::new(*pos, XY::new(width, 1));

            if old_rect.intersect(new_rect).is_some() {
                return false;
            }
        }

        true
    }

    fn internal_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        /* TODO
        Introduction of labels lowered the performance of this method significantly. Follow the standard protocol:
        1) create shitload of widget tests
        2) then optimise the function
         */

        let is_dropping_cursor = match &self.state {
            EditorState::Editing => false,
            EditorState::DroppingCursor { .. } => true,
        };

        let default = Self::get_cell_style(theme, CursorStatus::None, is_dropping_cursor, false, focused);
        helpers::fill_output(default.background, output);

        let buffer = unpack_unit!(self.buffer.lock(), "failed to lock buffer for rendering",);
        let cursor_set_copy = match buffer.cursors(self.wid) {
            None => {
                error!("failed to acquire cursor set for widget {}", self.wid);
                CursorSet::single()
            }
            Some(cs) => cs.clone(),
        };

        let visible_rect = output.visible_rect();

        let char_range_op = buffer.get_visible_chars_range(output);
        // highlights are actually just code coloring
        let highlights = buffer.highlight(char_range_op.clone());

        let mut highlight_iter = highlights.iter().peekable();
        let lines_to_skip = visible_rect.upper_left().y as usize;

        let mut lines_it = buffer.lines().skip(lines_to_skip);
        // skipping lines that cannot be visible, because they are before hint()
        let mut line_idx = lines_to_skip;

        // preparing labels
        // Right now labels "chain" one after another. Provided priority does not change, they should not
        // glitter.
        let mut labels: BTreeMap<XY, &Label> = BTreeMap::new();

        // if we don't have a char_range, that means the "visible rect" is empty, so we don't draw anything
        if let Some(char_range) = char_range_op {
            for label_provider in self.providers.todo_label_providers() {
                for label in label_provider.query_for(buffer.get_path()) {
                    if label
                        .pos
                        .maybe_should_draw(char_range.clone(), lines_to_skip..visible_rect.lower_right().y as usize)
                    {
                        if let Some(xy) = label.pos.into_position(&*buffer) {
                            if (xy.y as usize) < lines_to_skip {
                                continue;
                            }

                            if Self::can_add_label(&mut labels, (xy, label)) {
                                labels.insert(xy, label);
                            } else {
                                warn!("Discarding a label because of collision. This is an omission most likely.");
                            }
                        }
                    }
                }
            }
        }

        // for (line_idx, line) in self.buffer.new_lines().enumerate()
        //     // skipping lines that cannot be visible, because they are before hint()
        //     .skip(output.size_constraint().visible_hint().upper_left().y as usize)
        while let Some(line) = lines_it.next() {
            // skipping lines that cannot be visible, because the are after the hint()
            if line_idx >= visible_rect.lower_right().y as usize {
                // debug!("early exit 7");
                break;
            }

            let line_begin = match buffer.line_to_char(line_idx) {
                Some(begin) => begin,
                None => continue,
            };

            // let's generate the "combined line" of actual file and labels.
            let mut filtered_labels: Vec<(XY, &Label)> = Vec::new();

            for (label_pos, label) in labels.iter() {
                if label_pos.y as usize != line_idx {
                    continue;
                }

                // some paranoia checks in case somebody removes ordering accidentally
                if let Some(last_label) = filtered_labels.last() {
                    debug_assert!(last_label.0.x <= label_pos.x);
                }

                filtered_labels.push((*label_pos, label));
            }

            let mut combined_line: Vec<(TextStyle, String)> = Vec::new();

            {
                let mut label_it = filtered_labels.iter().peekable();
                let mut x_offset: usize = 0;

                for (c_idx, c) in line.graphemes().enumerate() {
                    let text_pos = XY::new(x_offset as u16, line_idx as u16);

                    while let Some((label_pos, label)) = label_it.peek() {
                        if *label_pos == text_pos {
                            for (style, grapheme) in label.contents(self.providers.theme()).styled_graphemes() {
                                // TODO crazy unoptimization
                                combined_line.push((*style, grapheme.to_string()));
                            }
                            label_it.next();
                        } else {
                            break;
                        }
                    }

                    // now I just move code from typical rendering here

                    let char_idx = line_begin + c_idx;

                    while let Some(item) = highlight_iter.peek() {
                        if char_idx >= item.char_end {
                            highlight_iter.next();
                        } else {
                            break;
                        }
                    }

                    // TODO optimise
                    let tr = if c == "\n" { NEWLINE.to_string() } else { c.to_string() };

                    let is_special_cursor: bool = if let EditorState::DroppingCursor { special_cursor } = &self.state {
                        special_cursor.get_cursor_status_for_char(char_idx) == CursorStatus::UnderCursor
                    } else {
                        false
                    };

                    let mut style = Self::get_cell_style(
                        theme,
                        cursor_set_copy.get_cursor_status_for_char(char_idx),
                        is_dropping_cursor,
                        is_special_cursor,
                        focused,
                    );

                    if tr != NEWLINE {
                        // TODO cleanup
                        if let Some(item) = highlight_iter.peek() {
                            if let Some(color) = theme.name_to_color(&item.identifier) {
                                style.set_foreground(color);
                            }
                        }
                    }

                    x_offset += tr.width();
                    combined_line.push((style, tr));

                    if x_offset as u16 >= visible_rect.lower_right().x {
                        debug!("early exit 6: character after visible rect");
                        break;
                    }
                }

                // DRAWING LABELS THAT ARE FOLLOWING THE LINE
                {
                    // coloring background between last character in line and begin of a label
                    let is_dropping_cursor = match &self.state {
                        EditorState::Editing => false,
                        EditorState::DroppingCursor { .. } => true,
                    };

                    let mut local_style = Self::get_cell_style(theme, CursorStatus::None, is_dropping_cursor, false, focused);

                    // I follow up on not drawn labels
                    for (label_pos, label) in label_it {
                        // moving cursor to right place
                        while x_offset < label_pos.x as usize {
                            combined_line.push((local_style, " ".to_string()));
                            x_offset += 1;
                        }

                        for (style, grapheme) in label.contents(self.providers.theme()).styled_graphemes() {
                            // TODO crazy unoptimization
                            combined_line.push((*style, grapheme.to_string()));
                        }
                    }
                }
            }

            // ok, at this point I consume the combined line
            let mut x_offset: usize = 0;
            for (style, grapheme) in combined_line.styled_graphemes() {
                let pos = XY::new(x_offset as u16, line_idx as u16);

                output.print_at(pos, *style, grapheme);
                x_offset += grapheme.width();
            }

            line_idx += 1;
            // TODO u16 overflow
            if line_idx as u16 >= visible_rect.lower_right().y {
                // debug!("early exit 5 : osc : {:?}, output : {:?}", output.size_constraint(), output);
                break;
            }
        }

        let one_beyond_limit = buffer.len_chars();
        let last_line = buffer.char_to_line(one_beyond_limit).unwrap(); //TODO
        let x_beyond_last = one_beyond_limit - buffer.line_to_char(last_line).unwrap(); //TODO

        let one_beyond_last_pos = XY::new(x_beyond_last as u16, last_line as u16);

        if one_beyond_last_pos < visible_rect.lower_right() {
            let style = Self::get_cell_style(&theme, CursorStatus::None, is_dropping_cursor, false, focused);

            output.print_at(one_beyond_last_pos, style, BEYOND);
        }
    }

    fn render_hover(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        if let Some((_, hover)) = self.requested_hover.as_ref() {
            let rect = unpack_unit_e!(self.last_hover_rect, "render hover before layout",);
            let mut sub_output = SubOutput::new(output, rect);
            match hover {
                EditorHover::Completion(completion) => completion.render(theme, focused, &mut sub_output),
                EditorHover::Context(context) => context.render(theme, focused, &mut sub_output),
            }
        }
    }

    /*
    Returns whether a completions were closed or not.
     */
    pub fn close_hover(&mut self) -> bool {
        if self.requested_hover.is_some() {
            self.requested_hover = None;
            true
        } else {
            false
        }
    }

    pub fn has_completions(&self) -> bool {
        if let Some((_, hover)) = self.requested_hover.as_ref() {
            matches!(hover, EditorHover::Completion(_))
        } else {
            false
        }
    }

    pub fn close_completions(&mut self) -> bool {
        if !self.has_completions() {
            error!("not closing completions - they are not open");
            return false;
        }

        self.requested_hover = None;
        true
    }

    pub fn request_completions(&mut self, buffer: &BufferState) {
        let cursor = unpack_unit!(
            buffer.cursors(self.wid).and_then(|c| c.as_single()),
            "not opening completions - cursor not single.",
        );
        let navcomp = unpack_unit!(self.navcomp.clone(), "not opening completions - navcomp not available.",);
        let stupid_cursor = unpack_unit_e!(
            StupidCursor::from_real_cursor(buffer, cursor).ok(),
            "failed converting cursor to lsp_cursor",
        );
        let path = unpack_unit_e!(buffer.get_path(), "path not available",);

        let trigger_op = {
            let nt = navcomp.completion_triggers(path);
            if nt.is_empty() {
                None
            } else {
                Some(nt)
            }
        };

        let hover_settings = self.get_cursor_related_hover_settings(buffer, trigger_op);
        // let tick_sender = navcomp.todo_navcomp_sender().clone();
        let promise_op = navcomp.completions(path.clone(), stupid_cursor, hover_settings.as_ref().and_then(|c| c.trigger.clone()));

        match (promise_op, hover_settings) {
            (Some(promise), Some(hover_settings)) => {
                let comp = CompletionWidget::new(promise).with_fuzzy(true);
                debug!("created completion: settings [{:?}]", &hover_settings);
                self.requested_hover = Some((hover_settings, EditorHover::Completion(comp)));
            }
            _ => {
                debug!("something missing - promise or hover settings");
            }
        }
    }

    // TODO merge with function above
    pub fn update_completions(&mut self, buffer: &BufferState) {
        let cursor = unpack_unit!(
            buffer.cursors(self.wid).and_then(|c| c.as_single()),
            "not opening completions - cursor not single.",
        );

        let cursor_pos = match self.get_single_cursor_screen_pos(buffer, cursor) {
            Some(c) => c,
            None => {
                debug!("cursor not single or not on screen");
                self.close_completions();
                return;
            }
        };
        let navcomp = match self.navcomp.as_ref() {
            Some(nc) => nc,
            None => {
                error!("failed to update completions - navcomp not found");
                self.close_completions();
                return;
            }
        };

        let path = match buffer.get_path() {
            Some(p) => p,
            None => {
                error!("failed to update completions - path not found");
                self.close_completions();
                return;
            }
        };

        let trigger_op = {
            let nt = navcomp.completion_triggers(path);
            if nt.is_empty() {
                None
            } else {
                Some(nt)
            }
        };

        let new_settings = match self.get_cursor_related_hover_settings(buffer, trigger_op) {
            Some(s) => s,
            None => {
                debug!("no good settings to update hover");
                self.close_completions();
                return;
            }
        };

        let close: bool = if let Some((settings, EditorHover::Completion(completion_widget))) = self.requested_hover.as_mut() {
            if Some(settings.anchor.y) != cursor_pos.widget_space.map(|xy| xy.y) {
                debug!("closing hover because cursor moved");
                true
            } else {
                let query = new_settings.substring.clone();
                *settings = new_settings;
                completion_widget.set_query_substring(query);
                false
            }
        } else {
            false
        };

        if close {
            self.close_completions();
        }
    }

    pub fn apply_completion_action(&mut self, buffer: &mut BufferState, completion_action: &CompletionAction) -> bool {
        if let Some((hover, _)) = self.requested_hover.as_ref() {
            if let Some(substring) = hover.substring.as_ref() {
                if !substring.is_empty() {
                    let len = substring.len();
                    debug!(
                        "removing [{}..{})",
                        hover.cursor_screen_position.cursor.a,
                        hover.cursor_screen_position.cursor.a + len
                    );
                    buffer.remove(hover.cursor_screen_position.cursor.a, hover.cursor_screen_position.cursor.a + len);
                }
            }

            let CompletionAction::Insert(to_insert) = completion_action;
            buffer.apply_common_edit_message(
                CommonEditMsg::Block(to_insert.clone()),
                self.wid,
                self.page_height() as usize,
                Some(self.providers.clipboard()),
            ); // TODO unnecessary clone
            self.close_hover();

            true
        } else {
            error!("can't apply completion, hover settings are not present");
            false
        }
    }

    // This is supposed to be called each time cursor is moved
    fn todo_after_cursor_moved(&mut self, buffer: &BufferState) {
        let cursor = unpack_unit!(
            buffer.cursors(self.wid).and_then(|c| c.as_single()),
            "not opening completions - cursor not single.",
        );

        let _path = unpack_unit!(buffer.get_path(), "no path set",);
        let _stupid_cursor = unpack_unit!(
            StupidCursor::from_real_cursor(buffer, cursor).ok(),
            "failed conversion to stupid cursor",
        );

        // TODO add support for scrachpad (path == None)

        // self.navcomp_symbol = self.navcomp.as_ref().map(|navcomp|
        //     navcomp.todo_get_symbol_at(path, stupid_cursor)
        // ).flatten();
    }

    fn after_content_changed(&self, buffer: &BufferState) {
        if let (Some(navcomp), Some(path)) = (&self.navcomp, buffer.get_path()) {
            let contents = buffer.text().rope().clone();
            navcomp.submit_edit_event(path, contents);
        }
    }

    fn layout_hover(&mut self, visible_rect: Rect) {
        let (hover_settings, hover) = unpack_unit!(self.requested_hover.as_mut());

        let mid_line = (visible_rect.pos.y + visible_rect.size.y) / 2;
        debug_assert!(hover_settings.anchor.y >= visible_rect.pos.y, "anchored above visible space");
        debug_assert!(
            hover_settings.anchor.y < visible_rect.lower_right().y,
            "anchored below visible space"
        );
        let above = hover_settings.anchor.y > mid_line;

        if let EditorHover::Completion(cw) = hover {
            if !cw.poll_results_should_draw() {
                debug!("withdrawing completion widget");
                self.requested_hover = None;
                return;
            }
        }

        if visible_rect.size.x < Self::MIN_HOVER_WIDTH {
            warn!("not enough space to draw hover");
            self.requested_hover = None;
            return;
        }

        let hover_rect: Option<Rect> = {
            let maxx = min(visible_rect.size.x, Self::MAX_HOVER_WIDTH);
            let maxy = if above {
                hover_settings.anchor.y - visible_rect.pos.y
            } else {
                visible_rect.lower_right().y - hover_settings.anchor.y - 1 // there was a drawing,
                                                                           // it should be OK.
            };

            let hover_size = XY::new(maxx, maxy);

            // this says "to the right, but not if that would mean going out of visible rect"
            let pos_x = if hover_size.x + hover_settings.anchor.x > visible_rect.lower_right().x {
                visible_rect.lower_right().x - (hover_size.x + hover_settings.anchor.x)
            } else {
                hover_settings.anchor.x
            };

            if above {
                // since it's *above* and higher bound is *exclusive*, no +-1 is needed here.
                let rect_pos = XY::new(pos_x, hover_settings.anchor.y - hover_size.y);
                let rect = Rect::new(rect_pos, hover_size);
                debug_assert!(rect.lower_right() <= visible_rect.lower_right(), "not drawing beyond visible rect");
                Some(rect)
            } else
            /*below*/
            {
                let rect_pos = XY::new(pos_x, hover_settings.anchor.y + 1);
                let rect = Rect::new(rect_pos, hover_size);
                debug_assert!(rect.lower_right() <= visible_rect.lower_right(), "not drawing beyond visible rect");
                Some(rect)
            }
        };

        if let Some(hover_rect) = hover_rect {
            self.last_hover_rect = Some(hover_rect);

            if let Some(parent_space_hover_rect_view) = visible_rect.intersect(hover_rect) {
                let hover_space_hover_visible_rect = parent_space_hover_rect_view.minus_shift(hover_rect.pos).unwrap(); // TODO
                hover
                    .get_widget_mut()
                    .layout(Screenspace::new(hover_rect.size, hover_space_hover_visible_rect));
            } else {
                error!("no intersection between hover_rect and visible rect");
            }
        } else {
            self.requested_hover = None;
        }
    }

    /*
    The BufferState is passed to avoid double-locking
     */
    pub fn show_usages(&self, buffer: &BufferState) -> Option<Box<dyn AnyMsg>> {
        let navcomp = unpack_or_e!(&self.navcomp, None, "can't show usages without navcomp");
        let cursor = unpack_or!(
            buffer.cursors(self.wid).and_then(|c| c.as_single()),
            None,
            "not opening completions - cursor not single."
        );
        let path = unpack_or!(buffer.get_path(), None, "no path set");
        let stupid_cursor = unpack_or!(
            StupidCursor::from_real_cursor(buffer, cursor).ok(),
            None,
            "failed conversion to stupid cursor"
        );

        let highlight = buffer.smallest_highlight(cursor.a);
        let symbol_op: Option<String> = highlight
            .as_ref()
            .and_then(|item| buffer.get_selected_chars(Selection::new(item.char_begin, item.char_end)).0);

        let symbol_desc: String = match (highlight, symbol_op) {
            (Some(type_), Some(item)) => {
                format!("Usages of {} \"{}\"", type_.identifier.as_ref(), item)
            }
            _ => "Usages of symbol:".to_string(),
        };

        let promise = unpack_or!(
            navcomp.get_symbol_usages(path, stupid_cursor),
            None,
            "failed retrieving usage symbol"
        );
        let wrapped_promise = WrappedSymbolUsagesPromise::new(symbol_desc, promise);

        MainViewMsg::FindReferences {
            promise_op: Some(wrapped_promise),
        }
        .someboxed()
    }

    /*
    The BufferState is passed to avoid double-locking
     */
    pub fn todo_go_to_definition(&mut self, buffer: &BufferState) -> Option<Box<dyn AnyMsg>> {
        let navcomp = unpack_or_e!(&self.navcomp, None, "can't show usages without navcomp");
        let cursor = unpack_or!(
            buffer.cursors(self.wid).and_then(|c| c.as_single()),
            None,
            "not opening completions - cursor not single."
        );
        let path = unpack_or!(buffer.get_path(), None, "no path set");
        let stupid_cursor = unpack_or!(
            StupidCursor::from_real_cursor(buffer, cursor).ok(),
            None,
            "failed conversion to stupid cursor"
        );
        let highlight = buffer.smallest_highlight(cursor.a);

        let symbol_op: Option<String> = highlight
            .as_ref()
            .and_then(|item| buffer.get_selected_chars(Selection::new(item.char_begin, item.char_end)).0);

        let symbol_desc: String = match (highlight, symbol_op) {
            (Some(type_), Some(item)) => {
                format!("Definition of {} \"{}\"", type_.identifier.as_ref(), item)
            }
            _ => "Definition of symbol:".to_string(),
        };

        None
    }
}

impl Widget for EditorWidget {
    fn id(&self) -> WID {
        self.wid
    }
    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }
    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn size_policy(&self) -> SizePolicy {
        SizePolicy::MATCH_LAYOUT
    }

    fn full_size(&self) -> XY {
        if let Some(lock) = self.buffer.lock() {
            lock.size()
        } else {
            error!("couldn't lock buffer to count");
            MIN_EDITOR_SIZE
        }
    }

    fn layout(&mut self, screenspace: Screenspace) {
        if self.layout_res != Some(screenspace) {
            debug!("changed size");

            if self.requested_hover.is_some() {
                warn!("closing hover because of resize - layout information got outdated");
                self.requested_hover = None;
            }
        }

        self.last_hover_rect = None;
        self.layout_res = Some(screenspace);
        self.layout_hover(screenspace.visible_rect());
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        let c = &self.providers.config().keyboard_config.editor;

        match (&self.state, &self.requested_hover, input_event) {
            (_, _, _) if self.ignore_input_altogether => {
                debug!("ignoring input because ignore_input_altogether = true");
                None
            }
            (&EditorState::Editing, None, InputEvent::KeyInput(key)) if key == c.enter_cursor_drop_mode => {
                EditorWidgetMsg::ToCursorDropMode.someboxed()
            }
            (&EditorState::DroppingCursor { .. }, None, InputEvent::KeyInput(key)) if key.keycode == Keycode::Esc => {
                EditorWidgetMsg::ToEditMode.someboxed()
            }
            (&EditorState::DroppingCursor { special_cursor }, None, InputEvent::KeyInput(key)) if key.keycode == Keycode::Enter => {
                debug_assert!(special_cursor.is_simple());

                EditorWidgetMsg::DropCursorFlip { cursor: special_cursor }.someboxed()
            }
            (&EditorState::Editing, None, InputEvent::KeyInput(key)) if !self.readonly && key == c.request_completions => {
                EditorWidgetMsg::RequestCompletions.someboxed()
            }
            (&EditorState::Editing, None, InputEvent::KeyInput(key)) if !self.readonly && key == c.reformat => {
                EditorWidgetMsg::Reformat.someboxed()
            }
            // TODO change to if let Some() when it's stabilized
            (&EditorState::DroppingCursor { .. }, None, InputEvent::KeyInput(key)) if key_to_edit_msg(key).is_some() => {
                let cem = key_to_edit_msg(key).unwrap();
                if !cem.is_editing() {
                    EditorWidgetMsg::DropCursorMove { cem }.someboxed()
                } else {
                    None
                }
            }
            (&EditorState::Editing, None, InputEvent::EverythingBarTrigger) => EditorWidgetMsg::RequestContextBar.someboxed(),
            (&EditorState::Editing, None, InputEvent::KeyInput(key)) if key_to_edit_msg(key).is_some() => {
                let cem = key_to_edit_msg(key).unwrap();
                if cem.is_editing() && self.readonly {
                    None
                } else {
                    EditorWidgetMsg::EditMsg(key_to_edit_msg(key).unwrap()).someboxed()
                }
            }
            _ => None,
        }
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("update {:?}, receives {:?}", self as &dyn Widget, &msg);
        return match msg.as_msg::<EditorWidgetMsg>() {
            None => {
                debug!("expected EditorWidgetMsg, got {:?}, passing through", msg);
                Some(msg)
            }
            Some(msg) => {
                let result = if let Some(mut buffer) = self.buffer.clone().lock_rw() {
                    match (&self.state, msg) {
                        (&EditorState::Editing, EditorWidgetMsg::EditMsg(cem)) => {
                            let page_height = self.page_height();
                            // page_height as usize is safe, since page_height is u16 and usize is larger.
                            let changed = buffer.apply_common_edit_message(
                                cem.clone(),
                                self.wid,
                                page_height as usize,
                                Some(self.providers.clipboard()),
                            );

                            // TODO this needs to happen only if CONTENTS changed, not if cursor positions changed
                            if changed {
                                self.after_content_changed(&buffer);

                                if self.has_completions() {
                                    self.update_completions(&buffer);
                                }
                            }

                            // TODO I might want to add directions upper left (for substraction) and lower right (for addition)
                            match cme_to_direction(cem) {
                                None => {}
                                Some(direction) => self.update_kite(&buffer, direction),
                            };

                            self.todo_after_cursor_moved(&buffer);

                            None
                        }
                        (&EditorState::Editing, EditorWidgetMsg::ToCursorDropMode) => {
                            // self.cursors.simplify(); //TODO I removed it, but I don't know why it was here in the first place
                            self.enter_dropping_cursor_mode(&buffer);
                            None
                        }
                        (&EditorState::DroppingCursor { .. }, EditorWidgetMsg::ToEditMode) => {
                            self.enter_editing_mode();
                            None
                        }
                        (&EditorState::DroppingCursor { special_cursor }, EditorWidgetMsg::DropCursorFlip { cursor }) => {
                            debug_assert!(special_cursor.is_simple());

                            let cursor_set = unpack_or!(buffer.cursors_mut(self.wid), None, "can't get cursor set");

                            if !cursor_set.are_simple() {
                                warn!("Cursors were supposed to be simple at this point. Recovering, but there was error.");
                                cursor_set.simplify();
                            }

                            let has_cursor = cursor_set.get_cursor_status_for_char(special_cursor.a) == CursorStatus::UnderCursor;

                            if has_cursor {
                                // We don't remove a single cursor, to not invalidate invariant
                                if cursor_set.len() > 1 {
                                    if !cursor_set.remove_by_anchor(special_cursor.a) {
                                        warn!("Failed to remove cursor by anchor {}, ignoring request", special_cursor.a);
                                    }
                                } else {
                                    debug!("Not removing a single cursor at {}", special_cursor.a);
                                }
                            } else if !cursor_set.add_cursor(*cursor) {
                                warn!("Failed to add cursor {:?} to set", cursor);
                            }

                            debug_assert!(cursor_set.check_invariant());

                            None
                        }
                        (&EditorState::DroppingCursor { special_cursor }, EditorWidgetMsg::DropCursorMove { cem }) => {
                            let mut set = CursorSet::singleton(special_cursor);
                            // TODO make sure this had no changing effect?
                            let height = self.page_height();
                            apply_common_edit_message(
                                cem.clone(),
                                &mut set,
                                &mut vec![],
                                &mut *buffer,
                                height as usize,
                                Some(self.providers.clipboard()),
                            );
                            self.state = EditorState::DroppingCursor {
                                special_cursor: set.as_single().unwrap(),
                            };
                            None
                        }
                        (&EditorState::Editing, EditorWidgetMsg::RequestCompletions) => {
                            self.request_completions(&buffer);
                            None
                        }
                        (&EditorState::Editing, EditorWidgetMsg::HoverClose) => {
                            self.requested_hover = None;
                            None
                        }
                        (&EditorState::Editing, EditorWidgetMsg::CompletionWidgetSelected(completion)) => {
                            self.apply_completion_action(&mut buffer, completion);
                            None
                        }
                        (&EditorState::Editing, EditorWidgetMsg::RequestContextBar) => {
                            self.todo_request_context_bar(&buffer);
                            None
                        }
                        (&EditorState::Editing, EditorWidgetMsg::Reformat) => {
                            self.reformat(&mut buffer);
                            None
                        }
                        (&EditorState::Editing, EditorWidgetMsg::ShowUsages) => {
                            self.requested_hover = None;
                            self.show_usages(&buffer)
                        }
                        (&EditorState::Editing, EditorWidgetMsg::GoToDefinition) => {
                            self.requested_hover = None;
                            self.todo_go_to_definition();
                            None
                        }
                        (editor_state, msg) => {
                            error!("Unhandled combination of editor state {:?} and msg {:?}", editor_state, msg);
                            None
                        }
                    }
                } else {
                    error!("can't update - failed to acquire rwlock");
                    None
                };

                result
            }
        };
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        self.get_hover_subwidget().map(|w| w.get(self))
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        self.get_hover_subwidget().map(|w| w.get_mut(self))
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        #[cfg(test)]
        {
            output.emit_metadata(crate::io::output::Metadata {
                id: self.wid,
                typename: self.typename().to_string(),
                // TODO I am not sure of that
                rect: Rect::from_zero(output.size()),
                focused,
            });
        }

        debug_assert!(self.last_hover_rect.is_some() == self.requested_hover.is_some());

        self.internal_render(theme, focused, output);
        self.render_hover(theme, focused, output);
    }

    fn kite(&self) -> XY {
        self.kite
    }
}

impl HasInvariant for EditorWidget {
    fn check_invariant(&self) -> bool {
        if let Some(lock) = self.buffer.lock() {
            if !lock.check_invariant() {
                return false;
            };

            if !lock.text().has_cursor_set_for(self.wid) {
                return false;
            }

            true
        } else {
            false
        }
    }
}
