use std::collections::{HashMap, HashSet};
use std::str::from_utf8;
use std::sync::{Arc, RwLock};

use log::error;

use crate::fs::path::SPath;
use crate::fs::read_error::ReadError;
use crate::gladius::providers::Providers;
use crate::primitives::has_invariant::HasInvariant;
use crate::text::buffer_state::BufferState;
use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
use crate::widgets::main_view::main_view::DocumentIdentifier;

/*
This is a provider of mapping DocumentIdentifier to buffer.
 */
pub struct BufferRegister {
    buffers: HashMap<DocumentIdentifier, BufferSharedRef>,
}

pub type BufferRegisterRef = Arc<RwLock<BufferRegister>>;

pub struct OpenResult {
    pub buffer_shared_ref: Result<BufferSharedRef, ReadError>,
    pub opened: bool,
}

impl BufferRegister {
    pub fn new() -> BufferRegister {
        BufferRegister { buffers: HashMap::new() }
    }

    pub fn get_id_from_path(&self, path: &SPath) -> Option<DocumentIdentifier> {
        self.buffers
            .keys()
            .find(|di| di.file_path.as_ref().map(|sp| sp == path).unwrap_or(false))
            .map(|c| c.clone())
    }

    pub fn get_buffer_ref_from_path(&self, path: &SPath) -> Option<BufferSharedRef> {
        self.get_id_from_path(path)
            .map(|id| self.buffers.get(&id).map(|r| r.clone()))
            .flatten()
    }

    pub fn get_buffer_ref_from_id(&self, document_identifier: &DocumentIdentifier) -> Option<BufferSharedRef> {
        self.buffers.get(document_identifier).cloned()
    }

    pub fn open_new_file(&mut self, providers: &Providers) -> BufferSharedRef {
        let doc_id = DocumentIdentifier::new_unique();

        let buffer_state = BufferState::full(Some(providers.tree_sitter().clone()), doc_id.clone());

        let bsr = BufferSharedRef::new_from_buffer(buffer_state);

        // saving for later

        self.buffers.insert(doc_id, bsr.clone());
        bsr
    }

    // TODO document
    pub fn open_file(&mut self, providers: &Providers, path: &SPath) -> OpenResult {
        if let Some(id) = self.get_id_from_path(path) {
            let bsr = self.buffers.get(&id).unwrap();
            OpenResult {
                buffer_shared_ref: Ok(bsr.clone()),
                opened: false,
            }
        } else {
            let buffer_bytes: Vec<u8> = match providers.fsf().blocking_read_entire_file(&path) {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!("failed to read {}, because {}", &path, e);
                    return OpenResult {
                        buffer_shared_ref: Err(e),
                        opened: false,
                    };
                }
            };

            let buffer_str = match from_utf8(&buffer_bytes) {
                Ok(s) => s,
                Err(e) => {
                    error!("failed loading file {}, because utf8 error {}", &path, e);
                    return OpenResult {
                        buffer_shared_ref: Err(ReadError::Utf8Error(e)),
                        opened: false,
                    };
                }
            };

            let doc_id = DocumentIdentifier::new_unique().with_file_path(path.clone());

            let buffer_state = BufferState::full(Some(providers.tree_sitter().clone()), doc_id.clone()).with_text(buffer_str);

            let bsr = BufferSharedRef::new_from_buffer(buffer_state);

            // saving for later

            self.buffers.insert(doc_id, bsr.clone());

            OpenResult {
                buffer_shared_ref: Ok(bsr),
                opened: true,
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'_ DocumentIdentifier, &'_ BufferSharedRef)> {
        self.buffers.iter()
    }
}

impl HasInvariant for BufferRegister {
    fn check_invariant(&self) -> bool {
        // no two references to the same file

        let seen_refs: HashSet<SPath> = HashSet::new();

        for document_identifier in self.buffers.keys() {
            if let Some(path) = &document_identifier.file_path {
                if seen_refs.get(path).is_some() {
                    error!("path {} found twice!", path);
                    return false;
                }
            }
        }

        true
    }
}
