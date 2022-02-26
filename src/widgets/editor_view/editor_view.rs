use std::rc::Rc;

use log::{error, warn};
use termion::event::Event::Key;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{AnyMsg, InputEvent, Keycode, Output, SizeConstraint, Theme, TreeSitterWrapper, Widget};
use crate::io::filesystem_tree::filesystem_front::FsfRef;
use crate::io::style::TextStyle;
use crate::layout::dummy_layout::DummyLayout;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::primitives::arrow::Arrow;
use crate::primitives::cursor_set::{CursorSet, CursorStatus};
use crate::primitives::cursor_set_rect::cursor_set_to_rect;
use crate::primitives::helpers;
use crate::primitives::rect::Rect;
use crate::primitives::xy::{XY, ZERO};
use crate::text::buffer::Buffer;
use crate::text::buffer_state::BufferState;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::common_edit_msgs::{apply_cme, cme_to_direction, CommonEditMsg, key_to_edit_msg};
use crate::widgets::editor_view::msg::EditorViewMsg;
use crate::widgets::fuzzy_search::fuzzy_search::FuzzySearchWidget;
use crate::widgets::fuzzy_search::mock_items_provider::mock::MockItemProvider;
use crate::widgets::save_file_dialog::save_file_dialog::SaveFileDialogWidget;

const MIN_EDITOR_SIZE: XY = XY::new(32, 10);

const NEWLINE: &'static str = "⏎";
const BEYOND: &'static str = "⇱";

pub struct EditorView {
    wid: WID,
    cursors: CursorSet,

    last_size: Option<XY>,

    todo_text: BufferState,

    anchor: XY,
    tree_sitter: Rc<TreeSitterWrapper>,

    fs: FsfRef,

    // TODO I should refactor that these two don't appear at the same time.
    save_file_dialog: Option<SaveFileDialogWidget>,
    fuzzy_search: Option<FuzzySearchWidget>,
}

impl EditorView {
    pub fn new(tree_sitter: Rc<TreeSitterWrapper>, fs: FsfRef) -> EditorView {
        EditorView {
            wid: get_new_widget_id(),
            cursors: CursorSet::single(),
            last_size: None,
            todo_text: BufferState::new(tree_sitter.clone()),
            anchor: ZERO,
            tree_sitter,
            fs,
            save_file_dialog: None,
            fuzzy_search: None,
        }
    }

    pub fn with_buffer(self, buffer: BufferState) -> Self {
        EditorView {
            todo_text: buffer,
            ..self
        }
    }

    // This updates the "anchor" of view to match the direction of editing. Remember, the scroll will
    // follow the "anchor" with least possible change.
    fn update_anchor(&mut self, last_move_direction: Arrow) {
        // TODO test
        let cursor_rect = cursor_set_to_rect(&self.cursors, &self.todo_text);
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

    fn internal_layout(&mut self, size: XY) -> Vec<WidgetIdRect> {
        if let Some(sd) = self.save_file_dialog.as_mut() {
            let layout = HoverLayout::new(
                &mut DummyLayout::new(self.wid, size),
                &mut LeafLayout::new(sd),
                Rect::new(XY::new(4, 4), XY::new(30, 15)), //TODO
            ).calc_sizes(size);

            return layout;
        }

        if let Some(fuzzy) = self.fuzzy_search.as_mut() {
            let layout = HoverLayout::new(
                &mut DummyLayout::new(self.wid, size),
                &mut LeafLayout::new(fuzzy),
                Rect::new(XY::new(4, 4), XY::new(30, 15)), //TODO
            ).calc_sizes(size);

            return layout;
        }

        let mut layout = DummyLayout::new(self.wid, size);
        layout.calc_sizes(size)
    }

    fn internal_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let fill_color = theme.default_background(focused);
        helpers::fill_output(fill_color, output);

        for (line_idx, line) in self.todo_text.lines().enumerate()
            // skipping lines that cannot be visible, because they are before hint()
            .skip(output.size_constraint().hint().upper_left().y as usize)
        {
            // skipping lines that cannot be visible, because larger than the hint()
            if line_idx >= output.size_constraint().hint().lower_right().y as usize {
                break;
            }

            let line_begin = match self.todo_text.line_to_char(line_idx) {
                Some(begin) => begin,
                None => continue,
            };

            let mut x_offset: usize = 0;
            for (c_idx, c) in line.graphemes(true).into_iter().enumerate() {
                let char_idx = line_begin + c_idx;
                let cursor_status = self.cursors.get_cursor_status_for_char(char_idx);
                let pos = XY::new(x_offset as u16, line_idx as u16);

                // TODO optimise
                let text = format!("{}", c);
                let tr = if c == "\n" { NEWLINE } else { text.as_str() };

                let x = self.todo_text.char_to_kind(char_idx);
                let fg_color = x.map(|s| theme.name_to_theme(s)).flatten();

                match cursor_status {
                    CursorStatus::None => {
                        let mut style = theme.default_text(focused);
                        match fg_color {
                            Some(fgc) => style = style.with_foreground(fgc),
                            None => {}
                        };

                        output.print_at(pos, style, tr);
                    }
                    CursorStatus::WithinSelection => {
                        output.print_at(pos, theme.selected_text(focused), tr);
                    }
                    CursorStatus::UnderCursor => {
                        output.print_at(pos, theme.cursor(), tr);
                    }
                }

                x_offset += tr.width();
            }
        }

        let one_beyond_limit = self.todo_text.len_chars();
        let last_line = self.todo_text.char_to_line(one_beyond_limit).unwrap();//TODO
        let x_beyond_last = one_beyond_limit - self.todo_text.line_to_char(last_line).unwrap(); //TODO

        let one_beyond_last_pos = XY::new(x_beyond_last as u16, last_line as u16);
        match self.cursors.get_cursor_status_for_char(one_beyond_limit) {
            CursorStatus::None => {}
            CursorStatus::WithinSelection => {
                output.print_at(one_beyond_last_pos, theme.default_text(true), BEYOND);
            }
            CursorStatus::UnderCursor => {
                output.print_at(one_beyond_last_pos, theme.cursor(), BEYOND);
            }
        }
    }
}

impl Widget for EditorView {
    fn id(&self) -> WID {
        self.wid
    }

    fn typename(&self) -> &'static str {
        "editor_view"
    }

    fn min_size(&self) -> XY {
        MIN_EDITOR_SIZE
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        let size = sc.hint().size;
        self.last_size = Some(size);

        size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            //TODO refactor the settings

            InputEvent::Tick => None,
            InputEvent::KeyInput(key) => {
                if key.modifiers.CTRL && key.keycode == Keycode::Char('s') {
                    return if key.modifiers.SHIFT == false {
                        Some(Box::new(EditorViewMsg::Save))
                    } else {
                        Some(Box::new(EditorViewMsg::SaveAs))
                    }
                }

                if key.modifiers.CTRL && key.keycode == Keycode::Char('h') {
                    return Some(Box::new(EditorViewMsg::Fuzzy))
                }

                return if key.modifiers.ALT == false {
                    match key_to_edit_msg(key) {
                        None => None,
                        Some(edit_msg) => Some(Box::new(EditorViewMsg::EditMsg(edit_msg)))
                    }
                } else {
                    None
                }
            }
        };
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        return match msg.as_msg::<EditorViewMsg>() {
            None => {
                warn!("expecetd EditorViewMsg, got {:?}", msg);
                None
            }
            Some(msg) => match msg {
                EditorViewMsg::EditMsg(cem) => {
                    let page_height = match self.last_size {
                        Some(xy) => xy.y,
                        None => {
                            error!("received {:?} before retrieving last_size, using {} as page_height instead", cem, MIN_EDITOR_SIZE.y);
                            MIN_EDITOR_SIZE.y
                        }
                    };

                    // page_height as usize is safe, since page_height is u16 and usize is larger.
                    let _noop = apply_cme(*cem, &mut self.cursors, &mut self.todo_text, page_height as usize);

                    match cme_to_direction(*cem) {
                        None => {}
                        Some(direction) => self.update_anchor(direction)
                    };

                    None
                }
                EditorViewMsg::SaveAs => {
                    self.fuzzy_search = None;

                    None
                }
                EditorViewMsg::Fuzzy => {
                    self.fuzzy_search = Some(FuzzySearchWidget::new(
                        |_| Some(Box::new(EditorViewMsg::FuzzyClose))
                    ).with_provider(
                        Box::new(MockItemProvider::new(30))
                    ));

                    None
                }
                EditorViewMsg::FuzzyClose => {
                    self.fuzzy_search = None;
                    None
                }
                _ => {
                    warn!("unhandled message {:?}", msg);
                    None
                }
            }
        };
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        self.internal_render(theme, focused, output);

        if let Some(sd) = self.save_file_dialog.as_ref() {
            sd.render(theme, focused, output);
        }

        if let Some(fs) = self.fuzzy_search.as_ref() {
            fs.render(theme, focused, output);
        }
    }

    fn anchor(&self) -> XY {
        self.anchor
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        if self.save_file_dialog.is_some() {
            return self.save_file_dialog.as_ref().map(|f| f as &dyn Widget)
        }

        if self.fuzzy_search.is_some() {
            return self.fuzzy_search.as_ref().map(|f| f as &dyn Widget)
        }

        None
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        if self.save_file_dialog.is_some() {
            return self.save_file_dialog.as_mut().map(|f| f as &mut dyn Widget);
        }

        if self.fuzzy_search.is_some() {
            return self.fuzzy_search.as_mut().map(|f| f as &mut dyn Widget);
        }

        None
    }
}