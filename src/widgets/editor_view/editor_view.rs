use std::path::PathBuf;
use std::rc::Rc;

use log::{error, warn};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{AnyMsg, InputEvent, Keycode, Output, SizeConstraint, Widget};
use crate::io::filesystem_tree::filesystem_front::FsfRef;
use crate::io::sub_output::SubOutput;
use crate::layout::dummy_layout::DummyLayout;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::primitives::arrow::Arrow;
use crate::primitives::cursor_set::CursorSet;
use crate::primitives::cursor_set_rect::cursor_set_to_rect;
use crate::primitives::helpers;
use crate::primitives::rect::Rect;
use crate::primitives::theme::Theme;
use crate::primitives::xy::{XY, ZERO};
use crate::text::buffer::Buffer;
use crate::text::buffer_state::BufferState;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::widget::any_msg::AsAny;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::common_edit_msgs::{apply_cme, cme_to_direction, key_to_edit_msg};
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

    fsf: FsfRef,

    // TODO I should refactor that these two don't appear at the same time.
    save_file_dialog: Option<SaveFileDialogWidget>,
    fuzzy_search: Option<FuzzySearchWidget>,

    // not sure if that should be a part of buffer or editor
    path: Option<PathBuf>,
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
            fsf: fs,
            save_file_dialog: None,
            fuzzy_search: None,
            path: None,
        }
    }

    pub fn with_buffer(self, buffer: BufferState) -> Self {
        EditorView {
            todo_text: buffer,
            ..self
        }
    }

    pub fn with_path(self, path: PathBuf) -> Self {
        Self {
            path: Some(path),
            ..self
        }
    }

    pub fn with_path_op(self, path_op: Option<PathBuf>) -> Self {
        Self {
            path: path_op,
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
            let rect = EditorView::get_hover_rect(size);
            let layout = HoverLayout::new(
                &mut DummyLayout::new(self.wid, size),
                &mut LeafLayout::new(sd),
                rect,
            ).calc_sizes(size);

            return layout;
        }

        if let Some(fuzzy) = self.fuzzy_search.as_mut() {
            let rect = EditorView::get_hover_rect(size);
            let layout = HoverLayout::new(
                &mut DummyLayout::new(self.wid, size),
                &mut LeafLayout::new(fuzzy),
                rect,
            ).calc_sizes(size);

            return layout;
        }

        let mut layout = DummyLayout::new(self.wid, size);
        layout.calc_sizes(size)
    }

    fn internal_render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let default = theme.default_text(focused);
        helpers::fill_output(default.background, output);

        let char_range_op = self.todo_text.char_range(output);
        let highlights = self.todo_text.highlight(char_range_op);
        let mut highlight_iter = highlights.iter().peekable();

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

                while let Some(item) = highlight_iter.peek() {
                    if char_idx >= item.char_end {
                        highlight_iter.next();
                    } else {
                        break;
                    }
                }

                let cursor_status = self.cursors.get_cursor_status_for_char(char_idx);
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

                theme.cursor_background(cursor_status).map(|bg| {
                    style = style.with_background(bg);
                });

                if !focused {
                    style.foreground = style.foreground.half();
                }

                output.print_at(pos, style, tr);

                x_offset += tr.width();
            }
        }

        let one_beyond_limit = self.todo_text.len_chars();
        let last_line = self.todo_text.char_to_line(one_beyond_limit).unwrap();//TODO
        let x_beyond_last = one_beyond_limit - self.todo_text.line_to_char(last_line).unwrap(); //TODO

        let one_beyond_last_pos = XY::new(x_beyond_last as u16, last_line as u16);
        let mut style = default;
        theme.cursor_background(self.cursors.get_cursor_status_for_char(one_beyond_limit)).map(|bg| {
            style = style.with_background(bg);
        });

        output.print_at(one_beyond_last_pos, style, BEYOND);
    }

    fn get_hover_rect(max_size: XY) -> Rect {
        let margin = max_size / 10;
        Rect::new(margin,
                  max_size - margin * 2,
        )
    }

    fn has_dialog(&self) -> bool {
        self.fuzzy_search.is_some() || self.save_file_dialog.is_some()
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

        self.internal_layout(size);

        size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        return match input_event {
            //TODO refactor the settings
            InputEvent::KeyInput(key) if key.modifiers.ctrl && key.keycode == Keycode::Char('s') => {
                Some(Box::new(EditorViewMsg::SaveAs))
            }
            InputEvent::KeyInput(key) if key.modifiers.ctrl && key.keycode == Keycode::Char('h') => {
                Some(Box::new(EditorViewMsg::Fuzzy))
            }
            InputEvent::KeyInput(key) if key_to_edit_msg(key).is_some() => {
                Some(Box::new(EditorViewMsg::EditMsg(key_to_edit_msg(key).unwrap())))
            }
            _ => None,
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
                EditorViewMsg::SaveAs |
                EditorViewMsg::Save => {
                    self.fuzzy_search = None;
                    let mut save_file_dialog = SaveFileDialogWidget::new(
                        self.fsf.clone(),
                    ).with_on_cancel(|_| {
                        EditorViewMsg::OnSaveAsCancel.someboxed()
                    });
                    self.path.as_ref().map(|p| save_file_dialog.set_path(&p));
                    self.save_file_dialog = Some(save_file_dialog);

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
                EditorViewMsg::OnSaveAsCancel => {
                    self.save_file_dialog = None;
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
        self.internal_render(theme, focused && !self.has_dialog(), output);

        if let Some(sd) = self.save_file_dialog.as_ref() {
            let rect = EditorView::get_hover_rect(output.size_constraint().hint().size);
            sd.render(theme, focused, &mut SubOutput::new(output, rect));
        }

        if let Some(fs) = self.fuzzy_search.as_ref() {
            let rect = EditorView::get_hover_rect(output.size_constraint().hint().size);
            fs.render(theme, focused, &mut SubOutput::new(output, rect));
        }
    }

    fn anchor(&self) -> XY {
        self.anchor
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        if self.save_file_dialog.is_some() {
            return self.save_file_dialog.as_ref().map(|f| f as &dyn Widget);
        }

        if self.fuzzy_search.is_some() {
            return self.fuzzy_search.as_ref().map(|f| f as &dyn Widget);
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