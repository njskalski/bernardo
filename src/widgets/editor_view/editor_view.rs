use std::path::PathBuf;
use std::rc::Rc;
use crate::{ConfigRef, FsfRef, TreeSitterWrapper};
use crate::experiments::clipboard::ClipboardRef;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::editor_widget::editor_widget::EditorWidget;
use crate::widgets::save_file_dialog::save_file_dialog::SaveFileDialogWidget;

enum EditorViewState {
    Simple,
    SaveFileDialog(SaveFileDialogWidget),
    Find,
    FindReplace,
}

pub struct EditorView {
    wid: WID,

    editor: EditorWidget,
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
        config : ConfigRef,
        tree_sitter : Rc<TreeSitterWrapper>,
        fsf : FsfRef,
        clipboard : ClipboardRef,
    ) -> Self{
        let editor = EditorWidget::new(config.clone(),
                                       tree_sitter,
                                       fsf.clone(),
                                       clipboard.clone());

        EditorView {
            wid: get_new_widget_id(),
            editor,
            fsf,
            config,
            state: EditorViewState::Simple,
            start_path: None
        }
    }
}