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
use crate::primitives::xy::XY;
use crate::text::buffer_state::BufferState;
use crate::widget::any_msg::AsAny;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::editor_view::msg::EditorViewMsg;
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::save_file_dialog::save_file_dialog::SaveFileDialogWidget;
use crate::widgets::with_scroll::WithScroll;

enum EditorViewState {
    Simple,
    SaveFileDialog(SaveFileDialogWidget),
    Find,
    FindReplace,
}

pub struct EditorView {
    wid: WID,

    editor: WithScroll<EditorWidget>,
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
        EditorView {
            editor: self.editor.mutate_internal(|b| b.with_buffer(buffer)),
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
        let mut layout: Box<dyn Layout> = match &mut self.state {
            EditorViewState::Simple => {
                Box::new(LeafLayout::new(&mut self.editor))
            }
            EditorViewState::SaveFileDialog(save_file_dialog) => {
                let rect = Self::get_hover_rect(size);
                Box::new(HoverLayout::new(
                    &mut DummyLayout::new(self.wid, size),
                    &mut LeafLayout::new(&mut self.editor),
                    rect,
                ))
            }
            // EditorViewState::Find => {}
            // EditorViewState::FindReplace => {}
            //TODO remove
            _ => DummyLayout::new(self.wid, size).boxed(),
        };

        let res = layout.calc_sizes(size);
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
        }
    }

    fn get_focused(&self) -> Option<&dyn Widget> {
        todo!()
    }

    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        todo!()
    }

    fn set_focused(&mut self, wid: WID) -> bool {
        todo!()
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        todo!()
    }

    fn anchor(&self) -> XY {
        todo!()
    }

    fn subwidgets_mut(&mut self) -> Box<dyn Iterator<Item=&mut dyn Widget> + '_> where Self: Sized {
        todo!()
    }

    fn subwidgets(&self) -> Box<dyn Iterator<Item=&dyn Widget> + '_> where Self: Sized {
        todo!()
    }

    fn get_subwidget(&self, wid: WID) -> Option<&dyn Widget> where Self: Sized {
        todo!()
    }

    fn get_subwidget_mut(&mut self, wid: WID) -> Option<&mut dyn Widget> where Self: Sized {
        todo!()
    }
}