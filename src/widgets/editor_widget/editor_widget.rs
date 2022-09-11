use std::cmp::min;
use std::rc::Rc;

use crossbeam_channel::TrySendError;
use futures::{FutureExt, TryFutureExt};
use futures::future::err;
use log::{debug, error, warn};
use lsp_types::Hover;
use streaming_iterator::StreamingIterator;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{AnyMsg, ConfigRef, InputEvent, Keycode, LangId, Output, selfwidget, SizeConstraint, subwidget, Widget};
use crate::config::theme::Theme;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::regex_search::FindError;
use crate::experiments::subwidget_pointer::SubwidgetPointer;
use crate::fs::fsf_ref::FsfRef;
use crate::fs::path::SPath;
use crate::io::sub_output::SubOutput;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::lsp_client::helpers::{get_lsp_text_cursor, LspTextCursor};
use crate::primitives::arrow::Arrow;
use crate::primitives::color::Color;
use crate::primitives::common_edit_msgs::{apply_cem, cme_to_direction, key_to_edit_msg};
use crate::primitives::cursor_set::{Cursor, CursorSet, CursorStatus};
use crate::primitives::cursor_set_rect::cursor_set_to_rect;
use crate::primitives::helpers;
use crate::primitives::helpers::fill_output;
use crate::primitives::rect::Rect;
use crate::primitives::xy::{XY, ZERO};
use crate::text::buffer::Buffer;
use crate::text::buffer_state::BufferState;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::w7e::handler::NavCompRef;
use crate::w7e::navcomp_group::NavCompTick;
use crate::w7e::navcomp_provider::Completion;
use crate::widget::any_msg::AsAny;
use crate::widget::complex_widget::{ComplexWidget, DisplayState};
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::editor_widget::completion_widget::CompletionWidget;
use crate::widgets::editor_widget::msg::EditorWidgetMsg;

const MIN_EDITOR_SIZE: XY = XY::new(32, 10);
const MAX_HOVER_WIDTH: u16 = 15;
const DEFAULT_HOVER_HEIGHT: u16 = 5;

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

enum EditorHover {
    Completion(CompletionWidget)
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

    state: EditorState,

    // This is completion or navigation
    hover: Option<(Rect, EditorHover)>,
    display_state: Option<DisplayState<Self>>,
}

impl EditorWidget {
    pub fn new(config: ConfigRef,
               tree_sitter: Rc<TreeSitterWrapper>,
               fsf: FsfRef,
               clipboard: ClipboardRef,
               navcomp: Option<NavCompRef>,
    ) -> EditorWidget {
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
            navcomp,
            hover: None,
            display_state: None,
        }
    }

    pub fn set_navcomp(&mut self, navcomp: Option<NavCompRef>) {
        self.navcomp = navcomp;
    }

    pub fn with_buffer(mut self, buffer: BufferState, navcomp_op: Option<NavCompRef>) -> Self {
        self.buffer = buffer;
        self.navcomp = navcomp_op;
        let contents = self.buffer.text().to_string();

        match (self.navcomp.clone(), self.buffer.get_file_front()) {
            (Some(navcomp), Some(spath)) => {
                navcomp.file_open_for_edition(spath, contents);
            }
            _ => {
                debug!("not starting navigation, because navcomp is some: {}, ff is some: {}",
                    self.navcomp.is_some(), self.buffer.get_file_front().is_some() )
            }
        }


        self
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

    pub fn auto_trigger_completion(&self) -> bool {
        if let Some(cursor) = self.cursors().as_single() {
            if let Some(navcomp) = &self.navcomp {
                let path = match self.buffer().get_file_front() {
                    None => {
                        warn!("unimplemented autocompletion for non-saved files");
                        return false;
                    }
                    Some(s) => s,
                };

                for symbol in navcomp.completion_triggers(path) {
                    if self.buffer().text().ends_with_at(cursor.a, &symbol) {
                        debug!("auto-trigger completion on symbol \"{}\"", &symbol);
                        return true;
                    }
                }
                false
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_single_cursor_screen_pos(&self) -> Option<XY> {
        let c = self.cursors().as_single()?;
        let lspc = get_lsp_text_cursor(self.buffer(), c).map_err(
            |e| {
                error!("failed mappint cursor to lsp-cursor")
            }).ok()?;
        let lspcxy = match lspc.to_xy() {
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
                if !sc.visible_hint().contains(lspcxy) {
                    warn!("cursor seems to be outside visible hint.");
                    return None;
                }

                lspcxy - sc.visible_hint().upper_left()
            }
        };

        debug!("cursor {:?} converted to {:?} positioned at {:?}", c, lspc, local_pos);
        debug_assert!(local_pos >= ZERO);
        debug_assert!(local_pos < self.last_size.unwrap().visible_hint().size);

        Some(local_pos)
    }

    pub fn get_cursor_related_hover(&self) -> Option<Rect> {
        let cursor_pos = match self.get_single_cursor_screen_pos() {
            Some(cp) => cp,
            None => {
                error!("can't position hover, no cursor local pos");
                return None;
            }
        };

        // TODO completely arbitrary choice
        let last_size = match self.last_size {
            None => {
                error!("requested hover before layour");
                return None;
            }
            Some(ls) => ls,
        };

        // if cursor is in upper part, we draw below cursor, otherwise above it
        let above = cursor_pos.y > (last_size.visible_hint().size.y / 2);
        let width = min(MAX_HOVER_WIDTH, last_size.visible_hint().size.x - cursor_pos.x);
        // TODO underflow
        let height = min(DEFAULT_HOVER_HEIGHT, (last_size.visible_hint().size.y / 2) - 1);

        if above {
            debug_assert!(cursor_pos.y > height);
            Some(Rect::xxyy(
                cursor_pos.x,
                cursor_pos.x + width,
                cursor_pos.y - 1, // TODO underflow
                cursor_pos.y - height,
            ))
        } else {
            debug_assert!(cursor_pos.y + height < last_size.visible_hint().size.y);
            Some(Rect::xxyy(
                cursor_pos.x,
                cursor_pos.x + width,
                cursor_pos.y + 1,
                cursor_pos.y + height,
            ))
        }
    }

    pub fn todo_update_completion(&mut self) {
        if let Some(cursor) = self.cursors().as_single() {
            if let Some(navcomp) = self.navcomp.clone() {
                if self.auto_trigger_completion() {

                    // TODO
                    let path = match self.buffer().get_file_front() {
                        None => {
                            warn!("unimplemented autocompletion for non-saved files");
                            return;
                        }
                        Some(s) => s.clone(),
                    };

                    let stupid_cursor = match get_lsp_text_cursor(self.buffer(), cursor) {
                        Ok(sc) => sc,
                        Err(e) => {
                            error!("failed converting cursor to lsp_cursor: {:?}", e);
                            return;
                        }
                    };

                    // TODO

                    let hover_rect = match self.get_cursor_related_hover() {
                        None => {
                            error!("no place to draw completions!");
                            return;
                        }
                        Some(r) => r,
                    };

                    let tick_sender = navcomp.todo_navcomp_sender().clone();
                    let promise = async move {
                        navcomp.completions(path, stupid_cursor).await
                    };

                    let promise = promise.shared();
                    let second_promise = promise.clone();
                    let second_promise = second_promise.then(move |_| {
                        // TODO parameters of LspTick
                        match tick_sender.try_send(NavCompTick::LspTick(LangId::RUST, 0)) {
                            Ok(_) => {
                                debug!("sent navcomp tick");
                            }
                            Err(e) => {
                                error!("failed sending navcomp tick: {:?}", e);
                            }
                        };
                        futures::future::ready(())
                    });

                    tokio::spawn(second_promise);

                    let comp = CompletionWidget::new(Box::new(promise));
                    self.hover = Some((hover_rect, EditorHover::Completion(comp)));
                    debug!("created completion");
                }
            }
        } else {
            self.hover = None;
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
        if self.hover.is_none() {
            return None;
        }

        Some(SubwidgetPointer::<Self>::new(Box::new(
            |s: &EditorWidget| {
                match s.hover.as_ref().unwrap() {
                    (_, EditorHover::Completion(comp)) => comp as &dyn Widget,
                }
            }
        ), Box::new(
            |s: &mut EditorWidget| {
                match s.hover.as_mut().unwrap() {
                    (_, EditorHover::Completion(comp)) => comp as &mut dyn Widget,
                }
            }
        )))
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
        self.last_size = Some(sc);
        self.complex_layout(sc)
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
            // TODO change to if let Some() when it's stabilized
            (&EditorState::DroppingCursor { .. }, InputEvent::KeyInput(key)) if key_to_edit_msg(key).is_some() => {
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

                    if changed {
                        match (&self.navcomp, self.buffer.get_file_front()) {
                            (Some(navcomp), Some(path)) => {
                                let contents = self.buffer.text().to_string();
                                navcomp.submit_edit_event(path, contents);
                            }
                            _ => {}
                        }

                        self.todo_update_completion();
                    }

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

impl ComplexWidget for EditorWidget {
    fn get_layout(&self, max_size: XY) -> Box<dyn Layout<Self>> {
        match &self.hover {
            None => Box::new(LeafLayout::new(selfwidget!(Self))),
            Some((rect, _)) => {
                Box::new(HoverLayout::new(
                    Box::new(LeafLayout::new(selfwidget!(Self))),
                    Box::new(LeafLayout::new(self.get_hover_subwidget().unwrap())),
                    *rect,
                ))
            }
        }
    }

    fn get_default_focused(&self) -> SubwidgetPointer<Self> {
        selfwidget!(Self)
    }

    fn set_display_state(&mut self, display_state: DisplayState<Self>) {
        self.display_state = Some(display_state);
    }

    fn get_display_state_op(&self) -> Option<&DisplayState<Self>> {
        self.display_state.as_ref()
    }

    fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
        self.display_state.as_mut()
    }

    /*
    now the reason I need to override this method highlights a shortcoming in my design I did not forsee:
    namely, that "complex widget" can't really point to itself without overriding at least complex_render,
    and then chances are something else will blow up.

    Now here's why:
    - if I don't override it, default implementation will call self.render(), causing infinite recursion, because
        this is where "complex_render" should be called from
    - if I move hover to say "editor view", editor widget will NOT appear on focus path between Hover and editorview,
        introducing error in input handling, that would have to be bypassed by routing input back down, which is heresy.
    - kosher way would be to introduce another layer between editor widget and editor view. It sounds the best, I just
        need to come up with a fkn name for it. Editor_and_hover_view? Who owns LSP then? Fuck, this get's complicated.

    It does beg a question whether "benedict" design is not superior, but I'd need a whiteboard to proove that,
    and I am sitting in a cheap shack in Jeriquaquara right now, so I'll run with this and see where it takes me.

    But if I am discussing it here, the primary argument *against* "benedict" is that it does not define well
    focus transfer - what happens when we want to move focus to button that doesn't exist yet (deferred focus):
    this process is inevitably risky. The item may fail to appear, and then we have to find a "general" focus
    in rather flat hierarchy. I can "get back to the caller" to ask for "default", and if "caller" ceased to exist
    continue up the tree asking for defaults. This *could* work.

    No, focus seems to naturally flow DOWN the tree. Maybe I should just lift the requirement to add "exactly
    one" focus path item? Maybe a single widget should be able to attach a chain of it's children?

    It almost seems like the issue is in putting layouts as not separate widgets.
     */
    fn complex_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        fill_output(theme.ui.non_focused.background, output);

        let mut focused_drawn = false;

        match self.get_display_state_op() {
            None => error!("failed rendering save_file_dialog without cached_sizes"),
            Some(ds) => {
                let focused_subwidget = ds.focused.get(self);

                for wwr in &ds.wwrs {
                    let sub_output = &mut SubOutput::new(output, *wwr.rect());
                    let widget = wwr.widget().get(self);
                    let subwidget_focused = focused && widget.id() == focused_subwidget.id();
                    if widget.id() != self.id() {
                        widget.render(theme,
                                      subwidget_focused,
                                      sub_output);
                    } else {
                        self.internal_render(theme, subwidget_focused, sub_output);
                    }

                    focused_drawn |= subwidget_focused;
                }
            }
        }

        if !focused_drawn {
            error!("a focused widget is not drawn in {} #{}!", self.typename(), self.id())
        }
    }
}

impl Drop for EditorWidget {
    fn drop(&mut self) {
        debug!("dropping editor widget for buffer : [{:?}]", self.buffer.get_file_front());

        match (&self.navcomp, self.buffer.get_file_front()) {
            (Some(navcomp), Some(spath)) => {
                debug!("shutting down navcomp.");
                navcomp.file_closed(spath);
            }
            _ => {
                debug!("not stoping navigation, because navcomp is some: {}, ff is some: {}",
                    self.navcomp.is_some(), self.buffer.get_file_front().is_some() )
            }
        }
    }
}