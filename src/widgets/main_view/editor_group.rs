use std::rc::Rc;
use log::error;
use crate::primitives::scroll::ScrollDirection;
use crate::{FsfRef, TreeSitterWrapper, Widget};
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::filename_to_language::filename_to_language;
use crate::fs::file_front::FileFront;
use crate::fs::filesystem_front::ReadError;
use crate::text::buffer_state::BufferState;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::with_scroll::WithScroll;

// This "class" was made separate to made borrow-checker realize, that it is not a violation of safety
// to borrow from it AND main_view mutably at the same time.

// Also, this is very much work in progress.
pub struct EditorGroup {
    editors: Vec<WithScroll<EditorView>>,
}

impl Default for EditorGroup {
    fn default() -> Self {
        Self {
            editors: Vec::new(),
        }
    }
}

impl EditorGroup {
    pub fn get(&self, idx: usize) -> Option<&WithScroll<EditorView>> {
        if idx > self.editors.len() {
            error!("requested non-existent editor {}", idx);
        }
        self.editors.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut WithScroll<EditorView>> {
        if idx > self.editors.len() {
            error!("requested non-existent mut editor {}", idx);
        }
        self.editors.get_mut(idx)
    }

    pub fn open_empty(&mut self, tree_sitter: Rc<TreeSitterWrapper>, fsf: FsfRef, clipboard: ClipboardRef) -> usize {
        self.editors.push(
            WithScroll::new(
                EditorView::new(tree_sitter, fsf, clipboard),
                ScrollDirection::Both,
            ).with_line_no()
        );

        let res = self.editors.len() - 1;

        res
    }

    // TODO is it on error escalation path after failed read?
    pub fn open_file(&mut self, tree_sitter: Rc<TreeSitterWrapper>, ff: FileFront, clipboard: ClipboardRef) -> Result<usize, ReadError> {
        let file_contents = ff.read_whole_file()?;
        let lang_id_op = filename_to_language(ff.path());

        self.editors.push(WithScroll::new(
            EditorView::new(tree_sitter.clone(), ff.fsf().clone(), clipboard)
                .with_buffer(
                    BufferState::new(tree_sitter)
                        .with_text_from_rope(file_contents, lang_id_op)
                        .with_file_front(ff.clone())
                ).with_path_op(
                ff.path().parent().map(|p|
                    ff.fsf().get_item(p)
                ).flatten().map(|f| f.path_rc().clone())
            ),
            ScrollDirection::Both,
        ).with_line_no()
        );

        let res = self.editors.len() - 1;

        Ok(res)
    }

    pub fn get_if_open(&self, ff: &FileFront) -> Option<usize> {
        for (idx, editor) in self.editors.iter().enumerate() {
            if let Some(cff) = editor.internal().buffer_state().get_file_front() {
                if cff == ff {
                    return Some(idx);
                }
            }
        }

        None
    }
}