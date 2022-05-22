use std::rc::Rc;
use log::{error, warn};
use crate::primitives::scroll::ScrollDirection;
use crate::{AnyMsg, ConfigRef, FsfRef, TreeSitterWrapper, Widget};
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::filename_to_language::filename_to_language;
use crate::fs::file_front::FileFront;
use crate::fs::filesystem_front::ReadError;
use crate::text::buffer_state::BufferState;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};
use crate::widgets::with_scroll::WithScroll;

// This "class" was made separate to made borrow-checker realize, that it is not a violation of safety
// to borrow from it AND main_view mutably at the same time.

// Also, this is very much work in progress.
pub struct EditorGroup {
    editors: Vec<WithScroll<EditorView>>,
    config: ConfigRef,
}

impl EditorGroup {
    pub fn new(config: ConfigRef) -> EditorGroup {
        EditorGroup {
            editors: Vec::new(),
            config,
        }
    }

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
                EditorView::new(self.config.clone(), tree_sitter, fsf, clipboard),
                ScrollDirection::Both,
            ).with_line_no()
        );

        let res = self.editors.len() - 1;

        res
    }

    // TODO is it on error escalation path after failed read?
    pub fn open_file(&mut self, tree_sitter: Rc<TreeSitterWrapper>, ff: FileFront, clipboard: ClipboardRef) -> Result<usize, ReadError> {
        let file_contents = ff.read_entire_file_to_rope()?;
        let lang_id_op = filename_to_language(ff.path());

        self.editors.push(WithScroll::new(
            EditorView::new(self.config.clone(), tree_sitter.clone(), ff.fsf().clone(), clipboard)
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

    pub fn get_buffer_list_provider(&self) -> Box<dyn ItemsProvider> {
        let mut free_id = 0 as u16;
        let mut buffer_descs: Vec<BufferDesc> = self.editors.iter().enumerate().map(|(idx, item)| {
            match item.internal().buffer_state().get_file_front() {
                None => {
                    free_id += 1;
                    BufferDesc::Unnamed { pos: idx, id: free_id }
                }
                Some(ff) => BufferDesc::File {
                    pos: idx,
                    ff: ff.clone(),
                }
            }
        }).collect();


        Box::new(
            BufferNamesProvider {
                descs: buffer_descs
            }
        )
    }

    pub fn len(&self) -> usize {
        self.editors.len()
    }
}

enum BufferDesc {
    // pos is position in editors vector
    File { pos: usize, ff: FileFront },
    /*
    id corresponds to display name, pos to position in EditorGroup.editors vector
     */
    Unnamed { pos: usize, id: u16 },
}

impl Item for BufferDesc {
    fn display_name(&self) -> &str {
        match self {
            BufferDesc::File { pos, ff } => ff.display_file_name(),
            BufferDesc::Unnamed { pos, id } => &format!("Unnamed #{}", id),
        }
    }

    fn comment(&self) -> Option<&str> {
        todo!()
    }

    fn on_hit(&self) -> Box<dyn AnyMsg> {
        todo!()
    }
}

pub struct BufferNamesProvider {
    descs: Vec<BufferDesc>,
}

impl ItemsProvider for BufferNamesProvider {
    fn context_name(&self) -> &str {
        "buffers"
    }

    fn items(&self, query: String, limit: usize) -> Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_> {
        todo!()
    }
}