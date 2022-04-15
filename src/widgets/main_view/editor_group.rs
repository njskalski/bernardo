use std::io;
use std::rc::Rc;
use log::error;
use crate::primitives::scroll::ScrollDirection;
use crate::{FsfRef, TreeSitterWrapper, Widget};
use crate::experiments::filename_to_language::filename_to_language;
use crate::fs::file_front::FileFront;
use crate::fs::filesystem_front::ReadError;
use crate::text::buffer_state::BufferState;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::no_editor::NoEditorWidget;
use crate::widgets::with_scroll::WithScroll;

// This "class" was made separate to made borrow-checker realize, that it is not a violation of safety
// to borrow from it AND main_view mutably at the same time.
pub struct EditorGroup {
    editors: Vec<WithScroll<EditorView>>,
    no_editor: NoEditorWidget,
    current: usize,
}

impl Default for EditorGroup {
    fn default() -> Self {
        Self {
            editors: Vec::new(),
            no_editor: NoEditorWidget::new(),
            current: 0,
        }
    }
}

impl EditorGroup {
    pub fn curr_editor(&self) -> &dyn Widget {
        if self.editors.len() == 0 {
            &self.no_editor
        } else {
            if self.current >= self.editors.len() {
                error!("current >= editors.len()");
                return &self.no_editor;
            }

            self.editors[self.current].internal()
        }
    }

    pub fn curr_editor_mut(&mut self) -> &mut dyn Widget {
        if self.editors.len() == 0 {
            &mut self.no_editor
        } else {
            if self.current >= self.editors.len() {
                error!("current >= editors.len(), fixing it, but that should not happen");
                self.current = 0;
            }

            self.editors[self.current].internal_mut()
        }
    }

    pub fn open_empty(&mut self, tree_sitter: Rc<TreeSitterWrapper>, fsf: FsfRef, set_current: bool) -> usize {
        self.editors.push(
            WithScroll::new(
                EditorView::new(tree_sitter, fsf),
                ScrollDirection::Both,
            )
        );

        let res = self.editors.len() - 1;

        if set_current {
            self.current = res;
        };

        res
    }

    // TODO is it on error escalation path after failed read?
    pub fn open_file(&mut self, tree_sitter: Rc<TreeSitterWrapper>, ff: FileFront, set_current: bool) -> Result<usize, ReadError> {
        let file_contents = ff.read_whole_file()?;
        let lang_id_op = filename_to_language(ff.path());

        self.editors.push(WithScroll::new(
            EditorView::new(tree_sitter.clone(), ff.fsf().clone())
                .with_buffer(
                    BufferState::new(tree_sitter)
                        .with_text_from_rope(file_contents, lang_id_op)
                ).with_path_op(
                ff.path().parent().map(|p|
                    ff.fsf().get_item(p)
                ).flatten().map(|f| f.path_rc().clone())
            ),
            ScrollDirection::Both,
        ));

        let res = self.editors.len() - 1;

        if set_current {
            self.current = res;
        };

        Ok(res)
    }
}