use std::cmp::min;
use std::rc::Rc;

use log::{debug, error, warn};
use matches::debug_assert_matches;
use streaming_iterator::StreamingIterator;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::config::config::ConfigRef;
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::regex_search::FindError;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::fsf_ref::FsfRef;
use crate::io::input_event::InputEvent;
use crate::io::keys::Keycode;
use crate::io::output::{Metadata, Output};
use crate::io::sub_output::SubOutput;
use crate::primitives::arrow::Arrow;
use crate::primitives::color::Color;
use crate::primitives::common_edit_msgs::{_apply_cem, cme_to_direction, CommonEditMsg, key_to_edit_msg};
use crate::primitives::cursor_set::{Cursor, CursorSet, CursorStatus, Selection};
use crate::primitives::cursor_set_rect::cursor_set_to_rect;
use crate::primitives::helpers;
use crate::primitives::rect::Rect;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::primitives::xy::XY;
use crate::promise::promise::Promise;
use crate::text::buffer_state::BufferState;
use crate::text::text_buffer::TextBuffer;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::w7e::handler::NavCompRef;
use crate::w7e::navcomp_provider::{CompletionAction, NavCompSymbol};
use crate::widget::any_msg::{AnyMsg, AsAny};
use crate::widget::widget::{get_new_widget_id, WID, Widget};
use crate::widgets::editor_widget::completion::completion_widget::CompletionWidget;
use crate::widgets::editor_widget::context_bar::widget::ContextBarWidget;
use crate::widgets::editor_widget::context_options_matrix::get_context_options;
use crate::widgets::editor_widget::helpers::{CursorPosition, find_trigger_and_substring};
use crate::widgets::editor_widget::msg::EditorWidgetMsg;

const MIN_EDITOR_SIZE: XY = XY::new(32, 10);
const MAX_HOVER_SIZE: XY = XY::new(64, 20);

const NEWLINE: &'static str = "⏎";
const BEYOND: &'static str = "⇱";

/*
This is heart and soul of Gladius Editor.

One important thing is that unless I come up with a great idea about scrolling, the "OverOutput"
has to be last output passed to render function, otherwise scrolling will fail to work.
 */

/*
TODO:
- display save error (and write tests)
- support for files with more than u16 lines
- I should probably remember "milestones" every several mb of file for big files.
- we have a lot of places where we check for Some on path, single cursor, navcomp and cast from cursor to stupid cursor.
    they need unification.
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
These settings are now generated in todo_ something and then *updated* in update_and_layout (after layouting the widget).
So modified in two places. Absolutely barbaric. To be changed.
 */
pub struct HoverSettings {
    pub rect: Rect,
    /*
     anchor is the character *in the line* (so it's never included in the rect). And it's
     - position of last character of trigger (if available)
     - just position of cursor
     */
    pub anchor: XY,
    pub cursor_position: CursorPosition,
    pub above_cursor: bool,
    pub substring: Option<String>,
    pub trigger: Option<String>,
}

enum EditorHover {
    Completion(CompletionWidget),
    Context(ContextBarWidget),
}

pub struct EditorWidget {
    wid: WID,

    last_size: Option<SizeConstraint>,

    buffer: BufferState,

    anchor: XY,
    tree_sitter: Rc<TreeSitterWrapper>,

    /*
    resist the urge to remove fsf from editor. It's used to facilitate "save as dialog".
    You CAN be working on two different filesystems at the same time, and save as dialog is specific to it.

    One thing to address is: "what if I have file from filesystem A, and I want to "save as" to B?". But that's beyond MVP, so I don't think about it now.
     */
    fsf: FsfRef,
    clipboard: ClipboardRef,
    config: ConfigRef,

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

    pub fn new(config: ConfigRef,
               tree_sitter: Rc<TreeSitterWrapper>,
               fsf: FsfRef,
               clipboard: ClipboardRef,
               navcomp: Option<NavCompRef>,
    ) -> EditorWidget {
        EditorWidget {
            wid: get_new_widget_id(),
            last_size: None,
            buffer: BufferState::full(tree_sitter.clone()),
            anchor: XY::ZERO,
            tree_sitter,
            fsf,
            config,
            clipboard,
            state: EditorState::Editing,
            navcomp,
            requested_hover: None,

            nacomp_symbol: None,
        }
    }

    pub fn set_navcomp(&mut self, navcomp: Option<NavCompRef>) {
        self.navcomp = navcomp;
    }

    pub fn with_buffer(mut self, buffer: BufferState, navcomp_op: Option<NavCompRef>) -> Self {
        self.buffer = buffer;
        self.navcomp = navcomp_op;
        let contents = self.buffer.text().rope.clone();

        match (self.navcomp.clone(), self.buffer.get_path()) {
            (Some(navcomp), Some(spath)) => {
                navcomp.file_open_for_edition(spath, contents);
            }
            _ => {
                debug!("not starting navigation, because navcomp is some: {}, ff is some: {}",
                    self.navcomp.is_some(), self.buffer.get_path().is_some() )
            }
        }

        self
    }

    // fn single_cursor_params(&self) -> (Option<String>, Option<String>) {
    //     if let Some(cur) = self.cursors().as_single() {
    //         self.navcomp.map(|navcomp| {
    //             navcomp.get
    //         })
    //
    //
    //     } else {
    //         None
    //     }
    // }

    // This updates the "anchor" of view to match the direction of editing. Remember, the scroll will
    // follow the "anchor" with least possible change.
    fn update_anchor(&mut self, last_move_direction: Arrow) {
        // TODO test
        // TODO cleanup - now cursor_set is part of buffer, we can move cursor_set_to_rect method there
        let cursor_rect = cursor_set_to_rect(&self.buffer.text().cursor_set, &self.buffer);
        match last_move_direction {
            Arrow::Up => {
                if self.anchor.y > cursor_rect.upper_left().y {
                    self.anchor.y = cursor_rect.upper_left().y;
                }
            }
            Arrow::Down => {
                if self.anchor.y < cursor_rect.lower_right().y {
                    self.anchor.y = cursor_rect.lower_right().y;
                }
            }
            Arrow::Left => {
                if self.anchor.x > cursor_rect.upper_left().x {
                    self.anchor.x = cursor_rect.upper_left().x;
                }
            }
            Arrow::Right => {
                if self.anchor.x < cursor_rect.lower_right().x {
                    self.anchor.x = cursor_rect.lower_right().x;
                }
            }
        }
    }

    pub fn enter_dropping_cursor_mode(&mut self) {
        debug_assert_matches!(self.state, EditorState::Editing);
        self.state = EditorState::DroppingCursor {
            special_cursor: self.cursors().iter().next().map(|c| *c).unwrap_or_else(|| {
                warn!("empty cursor set!");
                Cursor::single()
            })
        };
    }

    pub fn enter_editing_mode(&mut self) {
        debug_assert_matches!(self.state, EditorState::DroppingCursor { .. });
        self.state = EditorState::Editing;
    }

    pub fn get_single_cursor_screen_pos(&self, cursor: &Cursor) -> Option<CursorPosition> {
        let lsp_cursor = StupidCursor::from_real_cursor(self.buffer(), &cursor).map_err(
            |_| {
                error!("failed mapping cursor to lsp-cursor")
            }).ok()?;
        let lsp_cursor_xy = match lsp_cursor.to_xy(&self.buffer().text().rope) {
            None => {
                error!("lsp cursor beyond XY max");
                return None;
            }
            Some(x) => x,
        };

        let local_pos = match self.last_size {
            None => {
                error!("single_cursor_screen_pos called before first layout!");
                return None;
            }
            Some(sc) => {
                if !sc.visible_hint().contains(lsp_cursor_xy) {
                    warn!("cursor seems to be outside visible hint.");
                    return Some(CursorPosition {
                        cursor: *cursor,
                        screen_space: None,
                        absolute: lsp_cursor_xy,
                    });
                }

                lsp_cursor_xy - sc.visible_hint().upper_left()
            }
        };

        debug!("cursor {:?} converted to {:?} positioned at {:?}", cursor, lsp_cursor, local_pos);
        debug_assert!(local_pos >= XY::ZERO);
        debug_assert!(local_pos < self.last_size.unwrap().visible_hint().size);

        Some(CursorPosition {
            cursor: *cursor,
            screen_space: Some(local_pos),
            absolute: lsp_cursor_xy,
        })
    }

    /*
    triggers - if none, current cursor will become "anchor" +-1 line. But if it is provided,
        this function will stride left and aggregate substring between the cursor and ONE OF
        TRIGGERS (in order of apperance, only within visible space)

        I will use 1/2 height and all space right from cursor, capped by MAX_HOVER_SIZE.
     */
    pub fn get_cursor_related_hover_max(&self, triggers: Option<&Vec<String>>) -> Option<HoverSettings> {
        let last_size = match self.last_size {
            None => {
                error!("requested hover before layout");
                return None;
            }
            Some(ls) => {
                ls
            }
        };

        let cursor = match self.cursors().as_single() {
            None => {
                error!("multiple cursors or none, not doing hover");
                return None;
            }
            Some(c) => c,
        };

        let cursor_pos = match self.get_single_cursor_screen_pos(cursor) {
            Some(cp) => cp,
            None => {
                error!("can't position hover, no cursor local pos");
                return None;
            }
        };

        let cursor_screen_pos = match cursor_pos.screen_space {
            None => {
                error!("no cursor position in screen space");
                return None;
            }
            Some(local) => local,
        };

        // if cursor is in upper part, we draw below cursor, otherwise above it
        let above = cursor_screen_pos.y > (last_size.visible_hint().size.y / 2);
        // TODO underflows
        let width = min(MAX_HOVER_SIZE.x, last_size.visible_hint().size.x - cursor_screen_pos.x);
        let height = min(MAX_HOVER_SIZE.y, (last_size.visible_hint().size.y / 2) - 1);

        let trigger_and_substring: Option<(&String, String)> = triggers.map(|triggers| find_trigger_and_substring(
            triggers, &self.buffer, &cursor_pos)).flatten();

        let anchor = trigger_and_substring.as_ref().map(|tas| {
            let substr_width = tas.1.width() as u16; //TODO overflow
            if substr_width > cursor_screen_pos.x {
                debug!("absourd");
                cursor_screen_pos
            } else {
                cursor_screen_pos - XY::new(substr_width, 0)
            }
        }).unwrap_or(cursor_screen_pos);

        /*
         TODO the RECTs here are set twice, I never checked if they match. Maybe we should throw
              them away, or at least not set them here. Anyway, just so you know, fix it one day.
         */

        if above {
            debug_assert!(cursor_screen_pos.y > height);
            Some(HoverSettings {
                rect: Rect::new(anchor - XY::new(0, height), XY::new(width, height)),
                anchor,
                cursor_position: cursor_pos,
                above_cursor: true,
                substring: trigger_and_substring.as_ref().map(|tas| tas.1.clone()),
                trigger: trigger_and_substring.as_ref().map(|tas| tas.0.clone()),
            })
        } else {
            debug_assert!(cursor_screen_pos.y + height < last_size.visible_hint().size.y);
            Some(HoverSettings {
                rect: Rect::new(anchor + XY::new(0, 1), XY::new(width, height)),
                anchor,
                cursor_position: cursor_pos,
                above_cursor: false,
                substring: trigger_and_substring.as_ref().map(|tas| tas.1.clone()),
                trigger: trigger_and_substring.as_ref().map(|tas| tas.0.clone()),
            })
        }
    }

    fn todo_get_hover_settings_anchored_at_trigger(&self) -> Option<HoverSettings> {
        let path = match self.buffer().get_path() {
            None => {
                warn!("unimplemented autocompletion for non-saved files");
                return None;
            }
            Some(s) => s.clone(),
        };

        let navcomp = match self.navcomp.as_ref() {
            None => {
                error!("no navcomp available");
                return None;
            }
            Some(nc) => nc,
        };

        let trigger_op = {
            let nt = navcomp.completion_triggers(&path);
            if nt.is_empty() {
                None
            } else {
                Some(nt)
            }
        };

        let hover_settings = match self.get_cursor_related_hover_max(trigger_op) {
            None => {
                error!("no place to draw completions!");
                return None;
            }
            Some(r) => r,
        };

        Some(hover_settings)
    }

    pub fn todo_request_completion(&mut self) {
        if let Some(cursor) = self.cursors().as_single() {
            if let Some(navcomp) = self.navcomp.clone() {
                let stupid_cursor = match StupidCursor::from_real_cursor(self.buffer(), cursor) {
                    Ok(sc) => sc,
                    Err(e) => {
                        error!("failed converting cursor to lsp_cursor: {:?}", e);
                        return;
                    }
                };

                let path = match self.buffer.get_path() {
                    None => {
                        error!("path not available");
                        return;
                    }
                    Some(p) => p,
                };

                let hover_settings = self.todo_get_hover_settings_anchored_at_trigger();
                // let tick_sender = navcomp.todo_navcomp_sender().clone();
                let promise_op = navcomp.completions(path.clone(), stupid_cursor, hover_settings.as_ref().map(|c| c.trigger.clone()).flatten());

                match (promise_op, hover_settings) {
                    (Some(promise), Some(hover_settings)) => {
                        let comp = CompletionWidget::new(promise).with_fuzzy(true);
                        self.requested_hover = Some((hover_settings, EditorHover::Completion(comp)));
                        debug!("created completion");
                    }
                    _ => {
                        debug!("something missing - promise or hover settings");
                    }
                }
            } else {
                debug!("not opening completions - navcomp not available.")
            }
        } else {
            debug!("not opening completions - cursor not single.")
        }
    }

    pub fn todo_request_context_bar(&mut self) {
        debug!("request_context_bar");
        let single_cursor = self.cursors().as_single();
        let stupid_cursor_op = single_cursor.map(
            |c| StupidCursor::from_real_cursor(self.buffer(), c).ok()
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
            self.buffer.highlight(Some(range))
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
            self.cursors(),
            stupid_cursor_op,
            lsp_symbol_op,
            tree_sitter_highlight.as_ref().map(|c| c.as_str()));

        if items.is_empty() {
            warn!("ignoring everything bar, no items");
            self.requested_hover = None;
        } else {
            let hover_settings_op = self.get_cursor_related_hover_max(None);

            self.requested_hover = hover_settings_op.map(|hs| {
                let hover = EditorHover::Context(ContextBarWidget::new(items));
                (hs, hover)
            });
        }
    }

    fn pos_to_cursor(&self, theme: &Theme, char_idx: usize) -> Option<Color> {
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

        theme.cursor_background(self.cursors().get_cursor_status_for_char(char_idx))
    }

    pub fn page_height(&self) -> u16 {
        match self.last_size {
            Some(xy) => xy.visible_hint().size.y,
            None => {
                error!("requested height before layout, using {} as page_height instead", MIN_EDITOR_SIZE.y);
                MIN_EDITOR_SIZE.y
            }
        }
    }

    pub fn cursors(&self) -> &CursorSet {
        &self.buffer.text().cursor_set
    }

    pub fn buffer(&self) -> &BufferState {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut BufferState {
        &mut self.buffer
    }

    pub fn find_once(&mut self, phrase: &String) -> Result<bool, FindError> {
        let res = self.buffer.find_once(phrase);
        if res == Ok(true) {
            // TODO handle "restart from the top"
            self.update_anchor(Arrow::Down);
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

        let char_range_op = self.buffer.char_range(output);
        let highlights = self.buffer.highlight(char_range_op.clone());

        let mut highlight_iter = highlights.iter().peekable();

        let lines_to_skip = output.size_constraint().visible_hint().upper_left().y as usize;

        let mut lines_it = self.buffer.lines().skip(lines_to_skip);
        // skipping lines that cannot be visible, because they are before hint()
        let mut line_idx = lines_to_skip;

        // for (line_idx, line) in self.buffer.new_lines().enumerate()
        //     // skipping lines that cannot be visible, because they are before hint()
        //     .skip(output.size_constraint().visible_hint().upper_left().y as usize)
        while let Some(line) = lines_it.next() {
            // skipping lines that cannot be visible, because the are after the hint()
            if line_idx >= output.size_constraint().visible_hint().lower_right().y as usize {
                // debug!("early exit 7");
                break;
            }

            let line_begin = match self.buffer.line_to_char(line_idx) {
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

                self.pos_to_cursor(theme, char_idx).map(|mut bg| {
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
                if x_offset as u16 >= output.size_constraint().visible_hint().lower_right().x {
                    // debug!("early exit 6");
                    break;
                }
            }

            line_idx += 1;
            // TODO u16 overflow
            if line_idx as u16 >= output.size_constraint().visible_hint().lower_right().y {
                // debug!("early exit 5 : osc : {:?}, output : {:?}", output.size_constraint(), output);
                break;
            }
        }

        let one_beyond_limit = self.buffer.len_chars();
        let last_line = self.buffer.char_to_line(one_beyond_limit).unwrap();//TODO
        let x_beyond_last = one_beyond_limit - self.buffer.line_to_char(last_line).unwrap(); //TODO

        let one_beyond_last_pos = XY::new(x_beyond_last as u16, last_line as u16);

        if one_beyond_last_pos < output.size_constraint().visible_hint().lower_right() {
            let mut style = default;

            self.pos_to_cursor(theme, one_beyond_limit).map(|mut bg| {
                if !focused {
                    bg = bg.half();
                }
                style = style.with_background(bg);
            });

            output.print_at(one_beyond_last_pos, style, BEYOND);
        }
    }

    fn render_hover(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        if let Some((rect, hover)) = self.requested_hover.as_ref() {
            let mut sub_output = SubOutput::new(output, rect.rect);
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
        self.requested_hover.is_some()
    }

    /*
    If
     */
    pub fn update_completions(&mut self) {
        let cursor_pos = match self.cursors().as_single().map(|c| self.get_single_cursor_screen_pos(c)).flatten().clone() {
            Some(c) => c,
            None => {
                debug!("not found single cursor on screen");
                return;
            }
        };
        let new_settings = match self.todo_get_hover_settings_anchored_at_trigger() {
            Some(s) => s,
            None => {
                debug!("no good settings to update hover");
                self.close_hover();
                return;
            }
        };

        let close: bool = if let Some((settings, EditorHover::Completion(completion_widget))) = self.requested_hover.as_mut() {
            if Some(settings.anchor.y) != cursor_pos.screen_space.map(|xy| xy.y) {
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
            self.close_hover();
        }
    }

    pub fn apply_completion_action(&mut self, completion_action: &CompletionAction) -> bool {
        if let Some((hover, _)) = self.requested_hover.as_ref() {
            if let Some(substring) = hover.substring.as_ref() {
                if !substring.is_empty() {
                    let len = substring.len();
                    debug!("removing [{}..{})",hover.cursor_position.cursor.a, hover.cursor_position.cursor.a + len);
                    self.buffer.remove(hover.cursor_position.cursor.a, hover.cursor_position.cursor.a + len);
                }
            }

            let to_insert = match completion_action {
                CompletionAction::Insert(what) => what,
            };
            self.buffer.apply_cem(CommonEditMsg::Block(to_insert.clone()),
                                  self.page_height() as usize,
                                  Some(&self.clipboard)); // TODO unnecessary clone
            self.close_hover();

            true
        } else {
            error!("can't apply completion, hover settings are not present");
            false
        }
    }

    // This is supposed to be called each time cursor is moved
    fn todo_after_cursor_moved(&mut self) {
        // TODO add support for scrachpad (path == None)
        if let (Some(cursor), Some(path)) = (self.cursors().as_single(), self.buffer().get_path()) {
            if let Some(stupid_cursor) = StupidCursor::from_real_cursor(self.buffer(), cursor).ok() {
                self.nacomp_symbol = self.navcomp.as_ref().map(|navcomp|
                    navcomp.todo_get_symbol_at(path, stupid_cursor)
                ).flatten()
            }
        }
    }
}

impl Widget for EditorWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn min_size(&self) -> XY {
        MIN_EDITOR_SIZE
    }

    fn update_and_layout(&mut self, sc: SizeConstraint) -> XY {
        if self.last_size != Some(sc) {
            debug!("changed size");

            if self.requested_hover.is_some() {
                warn!("closing hover because of resize - layout information got outdated");
                self.requested_hover = None;
            }
        }

        self.last_size = Some(sc);

        let should_remove_hover = match self.requested_hover.as_mut() {
            None => {
                false
            }
            Some((rect, hover)) => match hover {
                EditorHover::Completion(comp) => {
                    if !comp.should_draw() {
                        true
                    } else {
                        let xy = comp.update_and_layout(SizeConstraint::simple(rect.rect.size));
                        if rect.above_cursor == false {
                            rect.rect.size = xy;
                            rect.rect.pos = rect.anchor + XY::new(0, 1);
                        } else {
                            rect.rect.size = xy;
                            // that's why we kept the anchor
                            // why no +-1? Because higher bounds are *exclusive*, like *everywhere* on the planet
                            rect.rect.pos = rect.anchor - XY::new(0, xy.y);
                        }
                        false
                    }
                }
                EditorHover::Context(context_bar) => {
                    let xy = context_bar.update_and_layout(SizeConstraint::simple(rect.rect.size));
                    // TODO Copy-pasted from above, reduce.
                    if rect.above_cursor == false {
                        rect.rect.size = xy;
                        rect.rect.pos = rect.anchor + XY::new(0, 1);
                    } else {
                        rect.rect.size = xy;
                        rect.rect.pos = rect.anchor - XY::new(0, xy.y);
                    }
                    false
                }
            }
        };
        if should_remove_hover {
            self.requested_hover = None;
        }

        sc.visible_hint().size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        let c = &self.config.keyboard_config.editor;
        return match (&self.state, input_event) {
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
            (&EditorState::Editing, InputEvent::KeyInput(key)) if key == c.request_completions => {
                EditorWidgetMsg::RequestCompletions.someboxed()
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
                EditorWidgetMsg::EditMsg(key_to_edit_msg(key).unwrap()).someboxed()
            }
            _ => None,
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        return match msg.as_msg::<EditorWidgetMsg>() {
            None => {
                warn!("expecetd EditorViewMsg, got {:?}", msg);
                None
            }
            Some(msg) => match (&self.state, msg) {
                (&EditorState::Editing, EditorWidgetMsg::EditMsg(cem)) => {
                    let page_height = self.page_height();
                    // page_height as usize is safe, since page_height is u16 and usize is larger.
                    let changed = self.buffer.apply_cem(cem.clone(), page_height as usize, Some(&self.clipboard));

                    // TODO this needs to happen only if CONTENTS changed, not if cursor positions changed
                    if changed {
                        match (&self.navcomp, self.buffer.get_path()) {
                            (Some(navcomp), Some(path)) => {
                                let contents = self.buffer.text().rope.clone();
                                navcomp.submit_edit_event(path, contents);
                            }
                            _ => {}
                        }

                        if self.has_completions() {
                            self.update_completions();
                        }
                    }

                    match cme_to_direction(cem) {
                        None => {}
                        Some(direction) => self.update_anchor(direction)
                    };

                    self.todo_after_cursor_moved();

                    None
                }
                (&EditorState::Editing, EditorWidgetMsg::ToCursorDropMode) => {
                    // self.cursors.simplify(); //TODO I removed it, but I don't know why it was here in the first place
                    self.enter_dropping_cursor_mode();
                    None
                }
                (&EditorState::DroppingCursor { .. }, EditorWidgetMsg::ToEditMode) => {
                    self.enter_editing_mode();
                    None
                }
                (&EditorState::DroppingCursor { special_cursor }, EditorWidgetMsg::DropCursorFlip { cursor }) => {
                    debug_assert!(special_cursor.is_simple());

                    if !self.buffer.text().cursor_set.are_simple() {
                        warn!("Cursors were supposed to be simple at this point. Recovering, but there was error.");
                        self.buffer.text_mut().cursor_set.simplify();
                    }

                    let has_cursor = self.buffer.text().cursor_set.get_cursor_status_for_char(special_cursor.a) == CursorStatus::UnderCursor;

                    if has_cursor {
                        let cs = &mut self.buffer.text_mut().cursor_set;

                        // We don't remove a single cursor, to not invalidate invariant
                        if cs.len() > 1 {
                            if !cs.remove_by_anchor(special_cursor.a) {
                                warn!("Failed to remove cursor by anchor {}, ignoring request", special_cursor.a);
                            }
                        } else {
                            debug!("Not removing a single cursor at {}", special_cursor.a);
                        }
                    } else {
                        if !self.buffer.text_mut().cursor_set.add_cursor(*cursor) {
                            warn!("Failed to add cursor {:?} to set", cursor);
                        }
                    }

                    debug_assert!(self.buffer.text().cursor_set.check_invariants());

                    None
                }
                (&EditorState::DroppingCursor { special_cursor }, EditorWidgetMsg::DropCursorMove { cem }) => {
                    let mut set = CursorSet::singleton(special_cursor);
                    // TODO make sure this had no changing effect?
                    let height = self.page_height();
                    _apply_cem(cem.clone(), &mut set, &mut self.buffer, height as usize, Some(&self.clipboard));
                    self.state = EditorState::DroppingCursor { special_cursor: *set.as_single().unwrap() };
                    None
                }
                (&EditorState::Editing, EditorWidgetMsg::RequestCompletions) => {
                    self.todo_request_completion();
                    None
                }
                (&EditorState::Editing, EditorWidgetMsg::HoverClose) => {
                    self.requested_hover = None;
                    None
                }
                (&EditorState::Editing, EditorWidgetMsg::CompletionWidgetSelected(completion)) => {
                    self.apply_completion_action(completion);
                    None
                }
                (&EditorState::Editing, EditorWidgetMsg::RequestContextBar) => {
                    self.todo_request_context_bar();
                    None
                }
                (editor_state, msg) => {
                    error!("Unhandled combination of editor state {:?} and msg {:?}", editor_state, msg);
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
        output.emit_metadata(
            Metadata {
                id: self.wid,
                typename: self.typename().to_string(),
                rect: output.size_constraint().visible_hint().clone(),
                focused,
            }
        );

        self.internal_render(theme, focused, output);
        self.render_hover(theme, focused, output);
    }

    fn anchor(&self) -> XY {
        self.anchor
    }
}

impl Drop for EditorWidget {
    fn drop(&mut self) {
        debug!("dropping editor widget for buffer : [{:?}]", self.buffer.get_path());

        match (&self.navcomp, self.buffer.get_path()) {
            (Some(_navcomp), Some(_spath)) => {
                debug!("shutting down navcomp.");
                // navcomp.file_closed(spath);
            }
            _ => {
                debug!("not stoping navigation, because navcomp is some: {}, ff is some: {}",
                    self.navcomp.is_some(), self.buffer.get_path().is_some() )
            }
        }
    }
}