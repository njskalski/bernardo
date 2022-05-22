use std::path::PathBuf;
use std::rc::Rc;
use crate::{AnyMsg, ConfigRef, FsfRef, InputEvent, Output, SizeConstraint, Theme, TreeSitterWrapper, Widget};
use crate::experiments::clipboard::ClipboardRef;
use crate::layout::dummy_layout::DummyLayout;
use crate::layout::layout::{Layout, WidgetIdRect};
use crate::layout::leaf_layout::LeafLayout;
use crate::primitives::scroll::ScrollDirection;
use crate::primitives::xy::XY;
use crate::widget::widget::{get_new_widget_id, WID};
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
            editor: WithScroll::new(editor, ScrollDirection::Vertical),
            fsf,
            config,
            state: EditorViewState::Simple,
            start_path: None,
        }
    }

    fn internal_layout(&mut self, sc: SizeConstraint) -> (XY, Vec<WidgetIdRect>) {
        let mut res: Vec<WidgetIdRect> = vec![];

        let mut layout: Box<dyn Layout> = match &mut self.state {
            EditorViewState::Simple => {
                LeafLayout::new(&mut self.editor).boxed()
            }
            // EditorViewState::SaveFileDialog(_) => {}
            // EditorViewState::Find => {}
            // EditorViewState::FindReplace => {}
            //TODO remove
            _ => DummyLayout::new(self.wid, sc.visible_hint().size).boxed(),
        };


        (sc.visible_hint().size, res)
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
        todo!()
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