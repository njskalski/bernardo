use std::cmp::{max, min};
use std::rc::Rc;
use std::sync::{Arc, RwLockReadGuard};

use log::{debug, error, log, warn};
use matches::debug_assert_matches;
use streaming_iterator::StreamingIterator;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{unpack_or, unpack_or_e};
use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::regex_search::FindError;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::fsf_ref::FsfRef;
use crate::gladius::providers::Providers;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::{Metadata, Output};
use crate::io::sub_output::SubOutput;
use crate::primitives::arrow::Arrow;
use crate::primitives::color::Color;
use crate::primitives::common_edit_msgs::{_apply_cem, cme_to_direction, CommonEditMsg, key_to_edit_msg};
use crate::primitives::cursor_set::{Cursor, CursorSet, CursorStatus};
use crate::primitives::cursor_set_rect::cursor_set_to_rect;
use crate::primitives::helpers;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::primitives::xy::XY;
use crate::promise::promise::{Promise, PromiseState};
use crate::text::buffer_state::BufferState;
use crate::text::text_buffer::TextBuffer;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::w7e::buffer_state_shared_ref::{BufferR, BufferSharedRef};
use crate::w7e::handler::NavCompRef;
use crate::w7e::navcomp_provider::{CompletionAction, NavCompSymbol};
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::editor_widget::completion::completion_widget::CompletionWidget;
use crate::widgets::editor_widget::context_bar::widget::ContextBarWidget;
use crate::widgets::editor_widget::context_options_matrix::get_context_options;
use crate::widgets::editor_widget::helpers::{CursorScreenPosition, find_trigger_and_substring};
use crate::widgets::editor_widget::msg::EditorWidgetMsg;
use crate::widgets::main_view::msg::MainViewMsg;

const MIN_EDITOR_SIZE: XY = XY::new(10, 3);
// const MAX_HOVER_SIZE: XY = XY::new(64, 20);

const NEWLINE: &'static str = "⏎";
const BEYOND: &'static str = "⇱";

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
    DroppingCursor {
        special_cursor: Cursor
    },
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

    last_size: Option<SizeConstraint>,
    // to be constructed in layout step based on HoverSettings
    last_hover_rect: Option<Rect>,

    buffer: BufferSharedRef,

    kite: XY,

    // navcomp is to submit edit messages, suggestion display will probably be somewhere else
    navcomp: Option<NavCompRef>,

    nacomp_symbol: Option<Box<dyn Promise<Option<NavCompSymbol>>>>,

    state: EditorState,

    // This is completion or navigation
    // Settings are calculated based on last_size, and entire hover will be discarded on resize.
    requested_hover: Option<(HoverSettings, EditorHover)>,
}

impl EditorWidget {
    pub const TYPENAME: &'static str = "editor_widget";

    const MAX_HOVER_WIDTH: u16 = 45;
    const MIN_HOVER_WIDTH: u16 = 15;

    pub fn new(providers: Providers,
               navcomp: Option<NavCompRef>,
               buffer_op: Option<BufferSharedRef>,
    ) -> EditorWidget {
        let tree_sitter_clone = providers.tree_sitter().clone();
        let buffer = buffer_op.unwrap_or(BufferSharedRef::new_empty(Some(tree_sitter_clone)));

        EditorWidget {
            wid: get_new_widget_id(),
            providers,
            readonly: false,
            ignore_input_altogether: false,
            last_size: None,
            last_hover_rect: None,
            buffer,
            kite: XY::ZERO,
            state: EditorState::Editing,
            navcomp,
            requested_hover: None,
            nacomp_symbol: None,

        }
    }

    pub fn with_readonly(self) -> Self {
        Self {
            readonly: true,
            ..self
        }
    }

    pub fn with_ignore_input_altogether(self) -> Self {
        Self {
            ignore_input_altogether: true,
            ..self
        }
    }

    pub fn set_navcomp(&mut self, navcomp: Option<NavCompRef>) {
        self.navcomp = navcomp;
    }

    pub fn get_buffer(&self) -> &BufferSharedRef {
        &self.buffer
    }

    //TODO have a feeling that navcomp can be merged with buffer
    pub fn set_buffer(&mut self, buffer: BufferSharedRef, navcomp_op: Option<NavCompRef>) {
        self.buffer = buffer;
        self.navcomp = navcomp_op;

        if let Some(buffer) = self.buffer.lock() {
            match (self.navcomp.clone(),
                   buffer.get_path().clone(),
            ) {
                (Some(navcomp), Some(spath)) => {
                    navcomp.file_open_for_edition(spath, buffer.text().rope.clone());
                }
                _ => {
                    debug!("not starting navigation, because navcomp is some: {}, ff is some: {}",
                            self.navcomp.is_some(), buffer.get_path().is_some() )
                }
            }
        } else {
            error!("failed locking buffer");
        }
    }

    // pub fn with_buffer(mut self, buffer: BufferSharedRef, navcomp_op: Option<NavCompRef>) -> Self {
    //     self.buffer = buffer;
    //     self.navcomp = navcomp_op;
    //     let buffer = unpack_or!(self.buffer.lock());
    //     let contents = buffer.text().rope.clone();
    //
    //     match (self.navcomp.clone(), self.buffer.get_path()) {
    //         (Some(navcomp), Some(spath)) => {
    //             navcomp.file_open_for_edition(spath, contents);
    //         }
    //         _ => {
    //             debug!("not starting navigation, because navcomp is some: {}, ff is some: {}",
    //                 self.navcomp.is_some(), self.buffer.get_path().is_some() )
    //         }
    //     }
    //
    //     self
    // }

    // This updates the "anchor" of view to match the direction of editing. Remember, the scroll will
    // follow the "anchor" with least possible change.
    fn update_kite(&mut self, buffer: &BufferState, last_move_direction: Arrow) {
        // TODO test
        // TODO cleanup - now cursor_set is part of buffer, we can move cursor_set_to_rect method there
        let cursor_rect = cursor_set_to_rect(&buffer.text().cursor_set, &*buffer);
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
        self.state = EditorState::DroppingCursor {
            special_cursor: buffer.cursors().iter().next().map(|c| *c).unwrap_or_else(|| {
                warn!("empty cursor set!");
                Cursor::single()
            })
        };
    }

    pub fn enter_editing_mode(&mut self) {
        debug_assert_matches!(self.state, EditorState::DroppingCursor { .. });
        self.state = EditorState::Editing;
    }

    pub fn get_single_cursor_screen_pos(&self, buffer: &BufferState, cursor: Cursor) -> Option<CursorScreenPosition> {
        let lsp_cursor = unpack_or_e!(StupidCursor::from_real_cursor(buffer, cursor).ok(), None, "failed mapping cursor to lsp-cursor");
        let lsp_cursor_xy = unpack_or_e!(lsp_cursor.to_xy(&buffer.text().rope), None, "lsp cursor beyond XY max");

        let sc = unpack_or!(self.last_size, None, "single_cursor_screen_pos called before first layout");
        let visible_rect = unpack_or!(sc.visible_hint(), None, "no visible rect - no screen cursor pos");

        if !visible_rect.contains(lsp_cursor_xy) {
            warn!("cursor seems to be outside visible hint {:?}", sc.visible_hint());
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
        let cursor: Cursor = unpack_or!(buffer.cursors().as_single().map(|c| c.clone()), None, "multiple cursors or none, not doing hover");
        let cursor_pos = unpack_or!(self.get_single_cursor_screen_pos(buffer, cursor), None, "can't position hover, no cursor local pos");
        let cursor_screen_pos = unpack_or!(cursor_pos.widget_space, None, "no cursor position in screen space");
        // let buffer_r: BufferR = unpack_or!(self.buffer.lock(), None, "failed to lock buffer");
        // let visible_rect = unpack_or!(last_size.visible_hint(), None, "no visible rect - no hover");


        let trigger_and_substring: Option<(&String, String)> = triggers
            .map(|triggers| find_trigger_and_substring(
                triggers, &*buffer, &cursor_pos)).flatten();

        let anchor = trigger_and_substring.as_ref().map(|tas| {
            let substr_width = tas.1.width() as u16; //TODO overflow
            if substr_width > cursor_screen_pos.x {
                debug!("absourd");
                cursor_screen_pos
            } else {
                cursor_screen_pos - XY::new(substr_width, 0)
            }
        }).unwrap_or(cursor_screen_pos);

        Some(HoverSettings {
            anchor,
            cursor_screen_position: cursor_pos,
            substring: trigger_and_substring.as_ref().map(|tas| tas.1.clone()),
            trigger: trigger_and_substring.as_ref().map(|tas| tas.0.clone()),
        })
    }


    pub fn todo_request_context_bar(&mut self, buffer: &BufferState) {
        debug!("request_context_bar");

        // need to resolve first
        if let Some(navcomp_symbol) = self.nacomp_symbol.as_mut() {
            navcomp_symbol.update();
        };

        let single_cursor = buffer.cursors().as_single();
        let stupid_cursor_op = single_cursor.map(
            |c| StupidCursor::from_real_cursor(buffer, c).ok()
        ).flatten();

        let lsp_symbol_op = self.nacomp_symbol.as_ref().map(|ncsp| {
            ncsp.read().map(|c| c.as_ref())
        }).flatten().flatten();

        let char_range_op = single_cursor.map(|a| {
            match a.s {
                None => {
                    a.a..a.a + 1
                }
                Some(sel) => {
                    sel.b..sel.e
                }
            }
        });

        // The reason I unpack and pack char_range_op, because I'm not interested in "all highlights"
        let tree_sitter_highlight = char_range_op.map(|range| {
            buffer.highlight(Some(range))
        }).map(|highlight_items| {
            // TODO I assume here that "first" is the smallest, it probably is not true
            // debug!("highlight items: [{:?}]", &highlight_items);
            highlight_items.first().map(|c| (*c).clone())
        }).flatten().map(|highlight_item| {
            highlight_item.identifier.to_string()
        });

        let items = get_context_options(
            &self.state,
            single_cursor,
            buffer.cursors(),
            stupid_cursor_op,
            lsp_symbol_op,
            tree_sitter_highlight.as_ref().map(|c| c.as_str()));

        if items.is_empty() {
            warn!("ignoring everything bar, no items");
            self.requested_hover = None;
        } else {
            let hover_settings_op = self.get_cursor_related_hover_settings(buffer, None);

            self.requested_hover = hover_settings_op.map(|hs| {
                let hover = EditorHover::Context(ContextBarWidget::new(items));
                (hs, hover)
            });
        }
    }

    pub fn reformat(&mut self, buffer: &mut BufferState) -> bool {
        let navcomp = unpack_or!(self.navcomp.as_ref(), false, "can't reformat: navcomp not available");
        let path = unpack_or!(buffer.get_path(), false , "can't reformat: unsaved file");
        let mut promise = unpack_or!(navcomp.todo_reformat(path), false, "can't reformat: no promise for reformat");

        if !buffer.text().cursor_set.are_simple() {
            warn!("can't format: unimplemented for non-simple cursors, sorry");
            return false;
        }

        if promise.wait() == PromiseState::Ready {
            // invariant : promise ready => take.is_some()
            let edits = unpack_or!(promise.read().unwrap(), false, "can't reformat: promise empty");

            let page_height = self.page_height();
            let res = buffer.apply_stupid_substitute_messages(edits, page_height as usize);

            // This theoretically could be optimised out, but maybe it's not worth it, it leads to
            // a new category of bugs if statement above turns out to be false, and it rarely is,
            // so it's very very hard to test. So I keep this here for peace of mind.
            self.after_content_changed(buffer);

            res
        } else {
            warn!("reformat promise broken");
            false
        }
    }

    fn pos_to_cursor(&self, cursors: &CursorSet, theme: &Theme, char_idx: usize) -> Option<Color> {
        match &self.state {
            EditorState::DroppingCursor { special_cursor } => {
                match special_cursor.get_cursor_status_for_char(char_idx) {
                    CursorStatus::UnderCursor => {
                        return Some(theme.ui.cursors.primary_anchor_background);
                    }
                    _ => {}
                }
            }
            _ => {}
        };

        theme.cursor_background(cursors.get_cursor_status_for_char(char_idx))
    }

    pub fn page_height(&self) -> u16 {
        match self.last_size {
            Some(sc) => {
                match sc.visible_hint() {
                    None => {
                        error!("requested page_height without screen space, using {} as page_height instead", MIN_EDITOR_SIZE.y);
                        MIN_EDITOR_SIZE.y
                    }
                    Some(xy) => {
                        xy.size.y
                    }
                }
            }
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

        buffer_mut.text_mut().cursor_set = cursor_set;
        self.update_kite(&buffer_mut, Arrow::Down);

        true
    }

    pub fn find_once(&mut self, phrase: &String) -> Result<bool, FindError> {
        let mut buffer = unpack_or_e!(self.buffer.lock_rw(), Err(FindError::FailedToLock), "failed locking buffer for search");

        let res = buffer.find_once(phrase);
        if res == Ok(true) {
            // TODO handle "restart from the top"
            self.update_kite(&buffer, Arrow::Down);
        }
        res
    }

    fn get_hover_subwidget(&self) -> Option<SubwidgetPointer<Self>> {
        if self.requested_hover.is_none() {
            return None;
        }

        Some(SubwidgetPointer::<Self>::new(Box::new(
            |s: &EditorWidget| {
                match s.requested_hover.as_ref().unwrap() {
                    (_, EditorHover::Completion(comp)) => comp as &dyn Widget,
                    (_, EditorHover::Context(cont)) => cont as &dyn Widget,
                }
            }
        ), Box::new(
            |s: &mut EditorWidget| {
                match s.requested_hover.as_mut().unwrap() {
                    (_, EditorHover::Completion(comp)) => comp as &mut dyn Widget,
                    (_, EditorHover::Context(cont)) => cont as &mut dyn Widget,
                }
            }
        )))
    }

    fn internal_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let default = match self.state {
            EditorState::Editing => { theme.default_text(focused) }
            EditorState::DroppingCursor { .. } => {
                theme.default_text(focused).with_background(theme.ui.mode_2_background)
            }
        };

        helpers::fill_output(default.background, output);

        let buffer = unpack_or!(self.buffer.lock(), (), "failed to lock buffer for rendering");

        let sc = output.size_constraint();
        let visible_rect = unpack_or!(sc.visible_hint(), (), "not visible - not rendering");

        let char_range_op = buffer.char_range(output);
        let highlights = buffer.highlight(char_range_op.clone());

        let mut highlight_iter = highlights.iter().peekable();

        let lines_to_skip = visible_rect.upper_left().y as usize;

        let mut lines_it = buffer.lines().skip(lines_to_skip);
        // skipping lines that cannot be visible, because they are before hint()
        let mut line_idx = lines_to_skip;

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

            let mut x_offset: usize = 0;
            for (c_idx, c) in line.graphemes(true).into_iter().enumerate() {
                let char_idx = line_begin + c_idx;

                while let Some(item) = highlight_iter.peek() {
                    if char_idx >= item.char_end {
                        highlight_iter.next();
                    } else {
                        break;
                    }
                }

                // let cursor_status = self.cursors.get_cursor_status_for_char(char_idx);
                let pos = XY::new(x_offset as u16, line_idx as u16);

                // TODO optimise
                let text = format!("{}", c);
                let tr = if c == "\n" { NEWLINE } else { text.as_str() };

                let mut style = default;

                if tr != NEWLINE { // TODO cleanup
                    if let Some(item) = highlight_iter.peek() {
                        if let Some(color) = theme.name_to_theme(&item.identifier) {
                            style = style.with_foreground(color);
                        }
                    }
                }

                self.pos_to_cursor(buffer.cursors(), theme, char_idx).map(|mut bg| {
                    if !focused {
                        bg = bg.half();
                    }
                    style = style.with_background(bg);
                });


                if !focused {
                    style.foreground = style.foreground.half();
                }

                output.print_at(pos, style, tr);

                x_offset += tr.width();
                if x_offset as u16 >= visible_rect.lower_right().x {
                    // debug!("early exit 6");
                    break;
                }
            }

            line_idx += 1;
            // TODO u16 overflow
            if line_idx as u16 >= visible_rect.lower_right().y {
                // debug!("early exit 5 : osc : {:?}, output : {:?}", output.size_constraint(), output);
                break;
            }
        }

        let one_beyond_limit = buffer.len_chars();
        let last_line = buffer.char_to_line(one_beyond_limit).unwrap();//TODO
        let x_beyond_last = one_beyond_limit - buffer.line_to_char(last_line).unwrap(); //TODO

        let one_beyond_last_pos = XY::new(x_beyond_last as u16, last_line as u16);

        if one_beyond_last_pos < visible_rect.lower_right() {
            let mut style = default;

            self.pos_to_cursor(buffer.cursors(), theme, one_beyond_limit).map(|mut bg| {
                if !focused {
                    bg = bg.half();
                }
                style = style.with_background(bg);
            });

            output.print_at(one_beyond_last_pos, style, BEYOND);
        }
    }

    fn render_hover(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        if let Some((_, hover)) = self.requested_hover.as_ref() {
            let rect = unpack_or_e!(self.last_hover_rect, (), "render hover before layout");
            let mut sub_output = SubOutput::new(output, rect);
            match hover {
                EditorHover::Completion(completion) => {
                    completion.render(theme, focused, &mut sub_output)
                }
                EditorHover::Context(context) => {
                    context.render(theme, focused, &mut sub_output)
                }
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
            match hover {
                EditorHover::Completion(_) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn close_completions(&mut self) -> bool {
        if self.has_completions() == false {
            error!("not closing completions - they are not open");
            return false;
        }

        self.requested_hover = None;
        true
    }

    pub fn request_completions(&mut self, buffer: &BufferState) {
        let cursor = unpack_or!(buffer.cursors().as_single(), (), "not opening completions - cursor not single.");
        let navcomp = unpack_or!(self.navcomp.clone(), (), "not opening completions - navcomp not available.");
        let stupid_cursor = unpack_or_e!(StupidCursor::from_real_cursor(buffer, cursor).ok(), (), "failed converting cursor to lsp_cursor");
        let path = unpack_or_e!(buffer.get_path(), (), "path not available");

        let trigger_op = {
            let nt = navcomp.completion_triggers(&path);
            if nt.is_empty() {
                None
            } else {
                Some(nt)
            }
        };

        let hover_settings = self.get_cursor_related_hover_settings(buffer, trigger_op);
        // let tick_sender = navcomp.todo_navcomp_sender().clone();
        let promise_op = navcomp.completions(path.clone(), stupid_cursor, hover_settings.as_ref().map(|c| c.trigger.clone()).flatten());

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
        let cursor_pos = match buffer.cursors().as_single().map(|c| self.get_single_cursor_screen_pos(buffer, c)).flatten().clone() {
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
            let nt = navcomp.completion_triggers(&path);
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
                    debug!("removing [{}..{})",hover.cursor_screen_position.cursor.a, hover.cursor_screen_position.cursor.a + len);
                    buffer.remove(hover.cursor_screen_position.cursor.a, hover.cursor_screen_position.cursor.a + len);
                }
            }

            let to_insert = match completion_action {
                CompletionAction::Insert(what) => what,
            };
            buffer.apply_cem(CommonEditMsg::Block(to_insert.clone()),
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
        let cursor = unpack_or!(buffer.cursors().as_single(), (), "cursor not single");
        let path = unpack_or!(buffer.get_path(), (), "no path set");
        let stupid_cursor = unpack_or!(StupidCursor::from_real_cursor(buffer, cursor).ok(), (), "failed conversion to stupid cursor");

        // TODO add support for scrachpad (path == None)

        self.nacomp_symbol = self.navcomp.as_ref().map(|navcomp|
            navcomp.todo_get_symbol_at(path, stupid_cursor)
        ).flatten();
    }

    fn after_content_changed(&self, buffer: &BufferState) {
        match (&self.navcomp, buffer.get_path()) {
            (Some(navcomp), Some(path)) => {
                let contents = buffer.text().rope.clone();
                navcomp.submit_edit_event(path, contents);
            }
            _ => {}
        }
    }

    fn layout_hover(&mut self, visible_rect: &Rect, _sc: SizeConstraint) {
        let (hover_settings, hover) = unpack_or!(self.requested_hover.as_mut(), ());

        let mid_line = (visible_rect.pos.y + visible_rect.size.y) / 2;
        debug_assert!(hover_settings.anchor.y >= visible_rect.pos.y, "anchored above visible space");
        debug_assert!(hover_settings.anchor.y < visible_rect.lower_right().y, "anchored below visible space");
        let above = hover_settings.anchor.y > mid_line;

        match hover {
            EditorHover::Completion(cw) => {
                if cw.poll_results_should_draw() == false {
                    debug!("withdrawing completion widget");
                    self.requested_hover = None;
                    return;
                }
            }
            _ => {}
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
                visible_rect.lower_right().y - hover_settings.anchor.y - 1 // there was a drawing, it should be OK.
            };

            let hover_sc = SizeConstraint::simple(XY::new(maxx, maxy));

            let hover_size = hover.get_widget_mut().layout(hover_sc);

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
            } else /*below*/ {
                let rect_pos = XY::new(pos_x, hover_settings.anchor.y + 1);
                let rect = Rect::new(rect_pos, hover_size);
                debug_assert!(rect.lower_right() <= visible_rect.lower_right(), "not drawing beyond visible rect");
                Some(rect)
            }
        };

        if hover_rect.is_none() {
            self.requested_hover = None;
        } else {
            self.last_hover_rect = hover_rect;
        }
    }

    pub fn show_usages(&self, buffer: &BufferState) -> Option<Box<dyn AnyMsg>> {
        let navcomp = unpack_or_e!(&self.navcomp, None, "can't show usages without navcomp");
        let cursor = unpack_or!(buffer.cursors().as_single(), None, "cursor not single");
        let path = unpack_or!(buffer.get_path(), None, "no path set");
        let stupid_cursor = unpack_or!(StupidCursor::from_real_cursor(buffer, cursor).ok(), None, "failed conversion to stupid cursor");

        let promise = navcomp.todo_get_symbol_usages(path, stupid_cursor);

        promise.map(|promise| MainViewMsg::FindReferences { promise_op: Some(promise) }.someboxed()).flatten()
    }

    pub fn todo_go_to_definition(&mut self) {}
}

impl Widget for EditorWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn size(&self) -> XY {
        MIN_EDITOR_SIZE
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        if self.last_size != Some(sc) {
            debug!("changed size");

            if self.requested_hover.is_some() {
                warn!("closing hover because of resize - layout information got outdated");
                self.requested_hover = None;
            }
        }

        self.last_hover_rect = None;

        self.last_size = Some(sc);
        let visible_rect = unpack_or!(sc.visible_hint(), self.size(), "can't layout greedy widget - no visible part");

        self.layout_hover(visible_rect, sc);

        visible_rect.size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        let c = &self.providers.config().keyboard_config.editor;

        return match (&self.state, input_event) {
            (_, _) if self.ignore_input_altogether => {
                debug!("ignoring input because ignore_input_altogether = true");
                None
            }
            (&EditorState::Editing, InputEvent::KeyInput(key)) if key == c.enter_cursor_drop_mode => {
                EditorWidgetMsg::ToCursorDropMode.someboxed()
            }
            (&EditorState::DroppingCursor { .. }, InputEvent::KeyInput(key)) if key.keycode == Keycode::Esc => {
                EditorWidgetMsg::ToEditMode.someboxed()
            }
            (&EditorState::DroppingCursor { special_cursor }, InputEvent::KeyInput(key)) if key.keycode == Keycode::Enter => {
                debug_assert!(special_cursor.is_simple());

                EditorWidgetMsg::DropCursorFlip { cursor: special_cursor }.someboxed()
            }
            (&EditorState::Editing, InputEvent::KeyInput(key)) if self.readonly == false && key == c.request_completions => {
                EditorWidgetMsg::RequestCompletions.someboxed()
            }
            (&EditorState::Editing, InputEvent::KeyInput(key)) if self.readonly == false && key == c.reformat => {
                EditorWidgetMsg::Reformat.someboxed()
            }
            // TODO change to if let Some() when it's stabilized
            (&EditorState::DroppingCursor { .. }, InputEvent::KeyInput(key)) if key_to_edit_msg(key).is_some() => {
                let cem = key_to_edit_msg(key).unwrap();
                if !cem.is_editing() {
                    EditorWidgetMsg::DropCursorMove { cem }.someboxed()
                } else {
                    None
                }
            }
            (&EditorState::Editing, InputEvent::EverythingBarTrigger) => {
                EditorWidgetMsg::RequestContextBar.someboxed()
            }
            (&EditorState::Editing, InputEvent::KeyInput(key)) if key_to_edit_msg(key).is_some() => {
                let cem = key_to_edit_msg(key).unwrap();
                if cem.is_editing() && self.readonly {
                    None
                } else {
                    EditorWidgetMsg::EditMsg(key_to_edit_msg(key).unwrap()).someboxed()
                }
            }
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        return match msg.as_msg::<EditorWidgetMsg>() {
            None => {
                debug!("expected EditorWidgetMsg, got {:?}, passing through", msg);
                Some(msg)
            }
            Some(msg) => {
                if let Some(mut buffer) = self.buffer.lock_rw() {
                    match (&self.state, msg) {
                        (&EditorState::Editing, EditorWidgetMsg::EditMsg(cem)) => {
                            let page_height = self.page_height();
                            // page_height as usize is safe, since page_height is u16 and usize is larger.
                            let changed = buffer.apply_cem(
                                cem.clone(),
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
                                Some(direction) => self.update_kite(&buffer, direction)
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

                            if !buffer.text().cursor_set.are_simple() {
                                warn!("Cursors were supposed to be simple at this point. Recovering, but there was error.");
                                buffer.text_mut().cursor_set.simplify();
                            }

                            let has_cursor = buffer.text().cursor_set.get_cursor_status_for_char(special_cursor.a) == CursorStatus::UnderCursor;

                            if has_cursor {
                                let cs = &mut buffer.text_mut().cursor_set;

                                // We don't remove a single cursor, to not invalidate invariant
                                if cs.len() > 1 {
                                    if !cs.remove_by_anchor(special_cursor.a) {
                                        warn!("Failed to remove cursor by anchor {}, ignoring request", special_cursor.a);
                                    }
                                } else {
                                    debug!("Not removing a single cursor at {}", special_cursor.a);
                                }
                            } else {
                                if !buffer.text_mut().cursor_set.add_cursor(*cursor) {
                                    warn!("Failed to add cursor {:?} to set", cursor);
                                }
                            }

                            debug_assert!(buffer.text().cursor_set.check_invariants());

                            None
                        }
                        (&EditorState::DroppingCursor { special_cursor }, EditorWidgetMsg::DropCursorMove { cem }) => {
                            let mut set = CursorSet::singleton(special_cursor);
                            // TODO make sure this had no changing effect?
                            let height = self.page_height();
                            _apply_cem(cem.clone(), &mut set, &mut *buffer, height as usize, Some(self.providers.clipboard()));
                            self.state = EditorState::DroppingCursor { special_cursor: set.as_single().unwrap() };
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
                            self.reformat(&mut *buffer);
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
                }
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
            let size = unpack_or!(output.size_constraint().visible_hint()).size;
            output.emit_metadata(
                Metadata {
                    id: self.wid,
                    typename: self.typename().to_string(),
                    // TODO I am not sure of that
                    rect: Rect::from_zero(size),
                    focused,
                }
            );
        }

        debug_assert!(self.last_hover_rect.is_some() == self.requested_hover.is_some());

        self.internal_render(theme, focused, output);
        self.render_hover(theme, focused, output);
    }

    fn kite(&self) -> XY {
        self.kite
    }
}
//
// impl Drop for EditorWidget {
//     fn drop(&mut self) {
//         debug!("dropping editor widget for buffer : [{:?}]", self.buffer.get_path());
//
//         match (&self.navcomp, self.buffer.get_path()) {
//             (Some(_navcomp), Some(_spath)) => {
//                 debug!("shutting down navcomp.");
//                 // navcomp.file_closed(spath);
//             }
//             _ => {
//                 debug!("not stoping navigation, because navcomp is some: {}, ff is some: {}",
//                     self.navcomp.is_some(), self.buffer.get_path().is_some() )
//             }
//         }
//     }
// }