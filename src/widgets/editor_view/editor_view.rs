use std::path::{Path, PathBuf};
use std::rc::Rc;
use log::{error, warn};
use crate::{AnyMsg, ConfigRef, FsfRef, InputEvent, Output, SizeConstraint, Theme, TreeSitterWrapper, Widget};
use crate::experiments::clipboard::ClipboardRef;
use crate::layout::dummy_layout::DummyLayout;
use crate::layout::hover_layout::HoverLayout;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::primitives::rect::Rect;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::xy;
use crate::primitives::xy::XY;
use crate::text::buffer_state::BufferState;
use crate::widget::any_msg::AsAny;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::edit_box::EditBoxWidget;
use crate::widgets::editor_view::msg::EditorViewMsg;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::save_file_dialog::save_file_dialog::SaveFileDialogWidget;
use crate::widgets::with_scroll::WithScroll;

#[derive(Copy)]
enum EditorFocus {
    Editor,
    Find,
    Replace,
}

enum EditorViewState {
    Simple,
    SaveFileDialog(SaveFileDialogWidget),
    Find(EditorFocus),
    FindReplace(EditorFocus),
}

pub struct EditorView {
    wid: WID,

    editor: WithScroll<EditorWidget>,
    find_box: EditBoxWidget,
    replace_box: EditBoxWidget,

    /*
    resist the urge to remove fsf from editor. It's used to facilitate "save as dialog".
    You CAN be working on two different filesystems at the same time, and save as dialog is specific to it.

    One thing to address is: "what if I have file from filesystem A, and I want to "save as" to B?". But that's beyond MVP, so I don't think about it now.
     */
    fsf: FsfRef,
    config: ConfigRef,

    state: EditorViewState,

    /*
    This represents "where the save as dialog should start", but only in case the file_front on buffer_state is None.
    If none, we'll use the fsf root.
    See get_save_file_dialog_path for details.
     */
    start_path: Option<Rc<PathBuf>>,
}

impl EditorView {
    pub fn new(
        config: ConfigRef,
        tree_sitter: Rc<TreeSitterWrapper>,
        fsf: FsfRef,
        clipboard: ClipboardRef,
    ) -> Self {
        let editor = EditorWidget::new(config.clone(),
                                       tree_sitter,
                                       fsf.clone(),
                                       clipboard.clone());
        EditorView {
            wid: get_new_widget_id(),
            editor: WithScroll::new(editor, ScrollDirection::Vertical).with_line_no(),
            find_box: EditBoxWidget::default(),
            replace_box: EditBoxWidget::default(),
            fsf,
            config,
            state: EditorViewState::Simple,
            start_path: None,
        }
    }

    pub fn with_path(self, path: Rc<PathBuf>) -> Self {
        Self {
            start_path: Some(path),
            ..self
        }
    }

    pub fn with_path_op(self, path_op: Option<Rc<PathBuf>>) -> Self {
        Self {
            start_path: path_op,
            ..self
        }
    }

    pub fn with_buffer(self, buffer: BufferState) -> Self {
        let editor = self.editor.mutate_internal(move |b| b.with_buffer(buffer));

        EditorView {
            editor,
            ..self
        }
    }

    fn get_hover_rect(max_size: XY) -> Rect {
        let margin = max_size / 10;
        Rect::new(margin,
                  max_size - margin * 2,
        )
    }

    fn internal_layout(&mut self, size: XY) -> Vec<WidgetIdRect> {
        let res = match &mut self.state {
            EditorViewState::Simple => {
                LeafLayout::new(&mut self.editor).calc_sizes(size)
            }
            EditorViewState::SaveFileDialog(save_file_dialog) => {
                let rect = Self::get_hover_rect(size);
                HoverLayout::new(
                    &mut DummyLayout::new(self.wid, size),
                    &mut LeafLayout::new(&mut self.editor),
                    rect,
                ).calc_sizes(size)
            }
            // EditorViewState::Find => {}
            // EditorViewState::FindReplace => {}
            //TODO remove
            _ => DummyLayout::new(self.wid, size).calc_sizes(size),
        };

        res
    }

    /*
    This attempts to save current file, but in case that's not possible (filename unknown) proceeds to open_save_as_dialog() below
     */
    fn save_or_save_as(&mut self) {
        let buffer = self.editor.internal().buffer();

        if let Some(ff) = buffer.get_file_front() {
            ff.overwrite_with(buffer);
        } else {
            self.open_save_as_dialog()
        }
    }

    fn open_save_as_dialog(&mut self) {
        match self.state {
            EditorViewState::Simple => {}
            _ => {
                warn!("open_save_as_dialog in unexpected state");
            }
        }

        let mut save_file_dialog = SaveFileDialogWidget::new(
            self.fsf.clone(),
        ).with_on_cancel(|_| {
            EditorViewMsg::OnSaveAsCancel.someboxed()
        }).with_on_save(|_, ff| {
            EditorViewMsg::OnSaveAsHit { ff }.someboxed()
        }).with_path(self.get_save_file_dialog_path());
    }

    fn positively_save_raw(&mut self, path: &Path) {
        let ff = match self.fsf.get_item(path) {
            None => {
                error!("attempted saving beyond root path");
                return;
            }
            Some(p) => p,
        };

        // setting the file path
        let buffer = self.editor.internal_mut().buffer_mut();
        buffer.set_file_front(Some(ff.clone()));

        // updating the "save as dialog" starting position
        ff.parent().map(|_f| {
            self.start_path = Some(ff.path_rc().clone())
        }).unwrap_or_else(|| {
            error!("failed setting save_as_dialog starting position - most likely parent is outside fsf root");
        });
    }

    /*
    This returns a (absolute) file path to be used with save_file_dialog. It can but does not have to
    contain filename part.
     */
    fn get_save_file_dialog_path(&self) -> &Rc<PathBuf> {
        let buffer = self.editor.internal().buffer();
        if let Some(ff) = buffer.get_file_front() {
            return ff.path_rc();
        };

        if let Some(sp) = self.start_path.as_ref() {
            return sp;
        }

        self.fsf.get_root_path()
    }

    fn set_state_simple(&mut self) {
        self.state = EditorViewState::Simple;
    }

    pub fn buffer_state(&self) -> &BufferState {
        self.editor.internal().buffer_state()
    }

    pub fn buffer_state_mut(&mut self) -> &mut BufferState {
        self.editor.internal_mut().buffer_state_mut()
    }

    fn focused_widget_from_focus(&self, focus: EditorFocus) -> &dyn Widget {
        match focus {
            EditorFocus::Editor => &self.editor as &dyn Widget,
            EditorFocus::Find => &self.find_box,
            EditorFocus::Replace => &self.replace_box,
        }
    }

    fn focused_widget_from_focus_mut(&mut self, focus: EditorFocus) -> &mut dyn Widget {
        match focus {
            EditorFocus::Editor => &mut self.editor as &mut dyn Widget,
            EditorFocus::Find => &mut self.find_box,
            EditorFocus::Replace => &mut self.replace_box,
        }
    }

    fn wid_to_focus(&self, wid: WID) -> Option<EditorFocus> {
        if self.editor.id() == wid {
            Some(EditorFocus::Editor)
        } else if self.find_box.id() == wid {
            Some(EditorFocus::Find)
        } else if self.replace_box.id() == wid {
            Some(EditorFocus::Replace)
        } else { None }
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
        XY::new(20, 8) // TODO completely arbitrary
    }

    fn layout(&mut self, sc: SizeConstraint) -> XY {
        todo!()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        return match msg.as_msg::<EditorViewMsg>() {
            None => {
                warn!("expecetd EditorViewMsg, got {:?}", msg);
                None
            }
            Some(msg) => match msg {
                EditorViewMsg::Save => {
                    self.save_or_save_as();
                    None
                }
                EditorViewMsg::SaveAs => {
                    self.open_save_as_dialog();
                    None
                }
                EditorViewMsg::OnSaveAsCancel => {
                    self.set_state_simple();
                    None
                }
                EditorViewMsg::OnSaveAsHit { ff } => {
                    // TODO handle errors
                    ff.overwrite_with(self.editor.internal().buffer());
                    self.set_state_simple();
                    None
                }
            }
        };
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        match &self.state {
            EditorViewState::Simple => Some(&self.editor),
            EditorViewState::SaveFileDialog(dialog) => Some(dialog),
            EditorViewState::Find(focus) => Some(self.focused_widget_from_focus(*focus)),
            EditorViewState::FindReplace(focus) => Some(self.focused_widget_from_focus(*focus))
        }
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        match &mut self.state {
            EditorViewState::Simple => Some(&mut self.editor),
            EditorViewState::SaveFileDialog(dialog) => Some(dialog),
            EditorViewState::Find(focus) => Some(self.focused_widget_from_focus_mut(*focus)),
            EditorViewState::FindReplace(focus) => Some(self.focused_widget_from_focus_mut(*focus))
        }
    }

    fn set_focused(&mut self, wid: WID) -> bool {
        match &self.state {
            EditorViewState::Simple => self.wid == wid,
            EditorViewState::SaveFileDialog(dialog) => dialog.id() == wid,
            EditorViewState::Find(focus) => {
                if self.editor.id() == wid {
                    self.state = EditorViewState::Find(EditorFocus::Editor);
                    true
                } else if self.find_box.id() == wid {
                    self.state = EditorViewState::Find(EditorFocus::Find);
                    true
                } else { false }
            }
            EditorViewState::FindReplace(focus) => {
                if self.editor.id() == wid {
                    self.state = EditorViewState::FindReplace(EditorFocus::Editor);
                    true
                } else if self.find_box.id() == wid {
                    self.state = EditorViewState::FindReplace(EditorFocus::Find);
                    true
                } else if self.replace_box.id() == wid {
                    self.state = EditorViewState::FindReplace(EditorFocus::Find);
                    true
                } else { false }
            }
        }
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let wirs = self.internal_layout(output.size_constraint().visible_hint().size);
    }

    fn anchor(&self) -> XY {
        xy::ZERO
    }

    fn subwidgets_mut(&mut self) -> Box<dyn Iterator<Item=&mut dyn Widget> + '_> where Self: Sized {
        let mut res: Vec<&mut dyn Widget> = vec![&mut self.editor];

        match &mut self.state {
            EditorViewState::Simple => {}
            EditorViewState::SaveFileDialog(dialog) => res.push(dialog),
            EditorViewState::Find(_) => res.push(&mut self.find_box),
            EditorViewState::FindReplace(_) => {
                res.push(&mut self.find_box);
                res.push(&mut self.replace_box);
            }
        };

        Box::new(res.into_iter())
    }

    fn subwidgets(&self) -> Box<dyn Iterator<Item=&dyn Widget> + '_> where Self: Sized {
        let mut res: Vec<&dyn Widget> = vec![&self.editor];

        match &self.state {
            EditorViewState::Simple => {}
            EditorViewState::SaveFileDialog(dialog) => res.push(dialog),
            EditorViewState::Find(_) => res.push(&self.find_box),
            EditorViewState::FindReplace(_) => {
                res.push(&self.find_box);
                res.push(&self.replace_box);
            }
        };

        Box::new(res.into_iter())
    }
}