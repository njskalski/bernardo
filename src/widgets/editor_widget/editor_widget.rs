use std::rc::Rc;

use log::{debug, error, warn};
use streaming_iterator::StreamingIterator;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{AnyMsg, ConfigRef, InputEvent, Keycode, Output, SizeConstraint, Widget};
use crate::experiments::clipboard::ClipboardRef;
use crate::fs::fsfref::FsfRef;
use crate::primitives::arrow::Arrow;
use crate::primitives::color::Color;
use crate::primitives::cursor_set::{Cursor, CursorSet, CursorStatus};
use crate::primitives::cursor_set_rect::cursor_set_to_rect;
use crate::primitives::helpers;
use crate::config::theme::Theme;
use crate::experiments::regex_search::FindError;
use crate::primitives::xy::{XY, ZERO};
use crate::text::buffer::Buffer;
use crate::text::buffer_state::BufferState;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::widget::any_msg::AsAny;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::primitives::common_edit_msgs::{apply_cem, cme_to_direction, key_to_edit_msg};
use crate::widgets::editor_widget::msg::EditorWidgetMsg;

const MIN_EDITOR_SIZE: XY = XY::new(32, 10);

const NEWLINE: &'static str = "⏎";
const BEYOND: &'static str = "⇱";

/*
TODO:
- display save error (and write tests)
- support for files with more than u16 lines
- I should probably remember "milestones" every several mb of file for big files.
 */

#[derive(Debug)]
enum EditorState {
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

pub struct EditorWidget {
    wid: WID,

    last_size: Option<XY>,

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

    state: EditorState,
}

impl EditorWidget {
    pub fn new(config: ConfigRef, tree_sitter: Rc<TreeSitterWrapper>, fsf: FsfRef, clipboard: ClipboardRef) -> EditorWidget {
        EditorWidget {
            wid: get_new_widget_id(),
            last_size: None,
            buffer: BufferState::new(tree_sitter.clone()),
            anchor: ZERO,
            tree_sitter,
            fsf,
            config,
            clipboard,
            state: EditorState::Editing,
        }
    }

    pub fn with_buffer(self, buffer: BufferState) -> Self {
        EditorWidget {
            buffer: buffer,
            ..self
        }
    }

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

    fn internal_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let default = match self.state {
            EditorState::Editing => { theme.default_text(focused) }
            EditorState::DroppingCursor { .. } => {
                theme.default_text(focused).with_background(theme.ui.mode_2_background)
            }
        };

        helpers::fill_output(default.background, output);

        let char_range_op = self.buffer.char_range(output);
        let highlights = self.buffer.highlight(char_range_op);
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
                    break;
                }
            }

            line_idx += 1;
            if line_idx as u16 >= output.size_constraint().visible_hint().lower_right().y {
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
            Some(xy) => xy.y,
            None => {
                error!("requested height before layout, using {} as page_height instead", MIN_EDITOR_SIZE.y);
                MIN_EDITOR_SIZE.y
            }
        }
    }

    pub fn buffer_state(&self) -> &BufferState {
        &self.buffer
    }

    pub fn buffer_state_mut(&mut self) -> &mut BufferState {
        &mut self.buffer
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
}

impl Widget for EditorWidget {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "editor_widget"
    }

    fn min_size(&self) -> XY {
        MIN_EDITOR_SIZE
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        let size = sc.visible_hint().size;
        self.last_size = Some(size);
        size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        let c = &self.config.keyboard_config.editor;
        return match (&self.state, input_event) {
            (&EditorState::Editing, InputEvent::KeyInput(key)) if key == c.enter_cursor_drop_mode => {
                EditorWidgetMsg::ToCursorDropMode.someboxed()
            }
            (&EditorState::DroppingCursor { special_cursor }, InputEvent::KeyInput(key)) if key.keycode == Keycode::Esc => {
                EditorWidgetMsg::ToEditMode.someboxed()
            }
            (&EditorState::DroppingCursor { special_cursor }, InputEvent::KeyInput(key)) if key.keycode == Keycode::Enter => {
                debug_assert!(special_cursor.is_simple());

                EditorWidgetMsg::DropCursorFlip { cursor: special_cursor }.someboxed()
            }
            // TODO change to if let Some() when it's stabilized
            (&EditorState::DroppingCursor { special_cursor }, InputEvent::KeyInput(key)) if key_to_edit_msg(key).is_some() => {
                let cem = key_to_edit_msg(key).unwrap();
                if !cem.is_editing() {
                    EditorWidgetMsg::DropCursorMove { cem }.someboxed()
                } else {
                    None
                }
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

                    match cme_to_direction(cem) {
                        None => {}
                        Some(direction) => self.update_anchor(direction)
                    };

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
                    apply_cem(cem.clone(), &mut set, &mut self.buffer, height as usize, Some(&self.clipboard));
                    self.state = EditorState::DroppingCursor { special_cursor: *set.as_single().unwrap() };
                    None
                }
                (editor_state, msg) => {
                    error!("Unhandled combination of editor state {:?} and msg {:?}", editor_state, msg);
                    None
                }
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        self.internal_render(theme, focused, output);
    }

    fn anchor(&self) -> XY {
        self.anchor
    }
}