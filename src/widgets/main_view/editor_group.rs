use std::rc::Rc;

use log::error;

use crate::{AnyMsg, ConfigRef,  TreeSitterWrapper};
use crate::experiments::beter_deref_str::BetterDerefStr;
use crate::experiments::clipboard::ClipboardRef;
use crate::experiments::filename_to_language::filename_to_language;
use crate::new_fs::fsf_ref::FsfRef;
use crate::new_fs::path::SPath;
use crate::new_fs::read_error::ReadError;
use crate::text::buffer_state::BufferState;
use crate::widgets::editor_view::editor_view::EditorView;
use crate::widgets::fuzzy_search::helpers::is_subsequence;
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};
use crate::widgets::main_view::msg::MainViewMsg;

// This "class" was made separate to made borrow-checker realize, that it is not a violation of safety
// to borrow from it AND main_view mutably at the same time.

// Also, this is very much work in progress.
pub struct EditorGroup {
    editors: Vec<EditorView>,
    config: ConfigRef,
}

impl EditorGroup {
    pub fn new(config: ConfigRef) -> EditorGroup {
        EditorGroup {
            editors: Vec::new(),
            config,
        }
    }

    pub fn get(&self, idx: usize) -> Option<&EditorView> {
        if idx > self.editors.len() {
            error!("requested non-existent editor {}", idx);
        }
        self.editors.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut EditorView> {
        if idx > self.editors.len() {
            error!("requested non-existent mut editor {}", idx);
        }
        self.editors.get_mut(idx)
    }

    pub fn open_empty(&mut self, tree_sitter: Rc<TreeSitterWrapper>, fsf: FsfRef, clipboard: ClipboardRef) -> usize {
        self.editors.push(
            EditorView::new(self.config.clone(), tree_sitter, fsf, clipboard),
        );

        let res = self.editors.len() - 1;

        res
    }

    // TODO is it on error escalation path after failed read?
    pub fn open_file(&mut self, tree_sitter: Rc<TreeSitterWrapper>, ff: SPath, clipboard: ClipboardRef) -> Result<usize, ReadError> {
        let file_contents = ff.read_entire_file_to_rope()?;
        let lang_id_op = filename_to_language(&ff);

        self.editors.push(
            EditorView::new(self.config.clone(), tree_sitter.clone(), ff.fsf().clone(), clipboard)
                .with_buffer(
                    BufferState::new(tree_sitter)
                        .with_text_from_rope(file_contents, lang_id_op)
                        .with_file_front(ff.clone())
                ).with_path_op(
                ff.parent()
            ),
        );

        let res = self.editors.len() - 1;

        Ok(res)
    }

    pub fn get_if_open(&self, ff: &SPath) -> Option<usize> {
        for (idx, editor) in self.editors.iter().enumerate() {
            if let Some(cff) = editor.buffer_state().get_file_front() {
                if cff == ff {
                    return Some(idx);
                }
            }
        }

        None
    }

    pub fn get_buffer_list_provider(&self) -> Box<dyn ItemsProvider> {
        let mut free_id = 0 as u16;
        let buffer_descs: Vec<BufferDesc> = self.editors.iter().enumerate().map(|(idx, item)| {
            match item.buffer_state().get_file_front() {
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

    pub fn is_empty(&self) -> bool {
        self.editors.is_empty()
    }
}

#[derive(Clone, Debug)]
enum BufferDesc {
    // pos is position in editors vector
    File { pos: usize, ff: SPath },
    /*
    id corresponds to display name, pos to position in EditorGroup.editors vector
     */
    Unnamed { pos: usize, id: u16 },
}

impl Item for BufferDesc {
    fn display_name(&self) -> BetterDerefStr {
        match self {
            BufferDesc::File { pos, ff } => {
                BetterDerefStr::Str(ff.last_name().unwrap_or("error getting filename"))
            },
            BufferDesc::Unnamed { pos, id } => BetterDerefStr::String(format!("Unnamed #{}", id)),
        }
    }

    fn comment(&self) -> Option<BetterDerefStr> {
        match self {
            BufferDesc::File { pos, ff } => {
                // TODO this is shit
                Some(BetterDerefStr::String(ff.display_name().as_ref_str().to_string()))
            },
            _ => None,
        }
    }

    fn on_hit(&self) -> Box<dyn AnyMsg> {
        match self {
            BufferDesc::File { pos, ff } => Box::new(MainViewMsg::FuzzyBuffersHit { pos: *pos }),
            BufferDesc::Unnamed { pos, id } => Box::new(MainViewMsg::FuzzyBuffersHit { pos: *pos }),
        }
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
        let mut items: Vec<BufferDesc> = vec![];

        for item in self.descs.iter() {
            if is_subsequence(item.display_name().as_ref_str(), &query) {
                items.push(item.clone());
                if items.len() >= limit {
                    break;
                }
            }
        }

        Box::new(items.into_iter().map(|b| Box::new(b) as Box<dyn Item>))
    }
}