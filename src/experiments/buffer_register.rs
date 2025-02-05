use std::collections::{HashMap, HashSet};
use std::str::from_utf8;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use log::{debug, error, warn};

use crate::fs::path::SPath;
use crate::fs::read_error::ReadError;
use crate::gladius::providers::Providers;
use crate::primitives::has_invariant::HasInvariant;
use crate::text::buffer_state::BufferState;
use crate::text::text_buffer::TextBuffer;
use crate::w7e::buffer_state_shared_ref::BufferSharedRef;
use crate::widgets::editor_widget::label::label::Label;
use crate::widgets::editor_widget::label::labels_provider::LabelsProvider;
use crate::widgets::main_view::main_view::BufferId;
use crate::widgets::main_view::main_view::DocumentIdentifier;

/*
This is a provider of mapping DocumentIdentifier to buffer.
 */
pub struct BufferRegister {
    buffers: HashMap<DocumentIdentifier, BufferSharedRef>,

    // When a BufferState is dropped, it's identifier will be sent for clearing. If it's not received, an error will be emitted.
    debug_channel: (Sender<DocumentIdentifier>, Receiver<DocumentIdentifier>),
}

const DEBUG_CHANNEL_DEADLINE: Duration = Duration::from_millis(600);

pub type BufferRegisterRef = Arc<RwLock<BufferRegister>>;

pub struct OpenResult {
    pub buffer_shared_ref: Result<BufferSharedRef, ReadError>,
    pub opened: bool,
}

impl BufferRegister {
    pub fn new() -> BufferRegister {
        let debug_channel = crossbeam_channel::unbounded::<DocumentIdentifier>();
        BufferRegister {
            buffers: HashMap::new(),
            debug_channel,
        }
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

        let buffer_state = BufferState::full(
            Some(providers.tree_sitter().clone()),
            doc_id.clone(),
            Some(self.debug_channel.0.clone()),
            providers.config().global.tabs_to_spaces,
        );

        let bsr = BufferSharedRef::new_from_buffer(buffer_state);

        // saving for later

        self.buffers.insert(doc_id, bsr.clone());
        bsr
    }

    pub fn are_any_buffers_unsaved(&self) -> bool {
        for (_, bsr) in self.buffers.iter() {
            if let Some(lock) = bsr.lock() {
                if lock.is_saved() == false {
                    return true;
                }
            }
        }

        false
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

            let mut buffer_state = BufferState::full(
                Some(providers.tree_sitter().clone()),
                doc_id.clone(),
                Some(self.debug_channel.0.clone()),
                providers.config().global.tabs_to_spaces,
            )
            .with_text(buffer_str)
            .with_maked_as_saved();

            if providers.config().global.guess_indent {
                buffer_state.guess_formatting_whitespace();
            }

            let bsr = BufferSharedRef::new_from_buffer(buffer_state);

            // saving for later

            self.buffers.insert(doc_id, bsr.clone());

            OpenResult {
                buffer_shared_ref: Ok(bsr),
                opened: true,
            }
        }
    }

    pub fn close_buffer(&mut self, document_identifier: &DocumentIdentifier) -> bool {
        /*
        This method is a little paranoid over-protected, because leaking references to buffers would
        result in a catastrophic memory leaks - the structures employed for code highlighting and
        history will be the largest objects in memory.
         */

        if self.buffers.contains_key(document_identifier) {
            let _removed = self.buffers.remove(document_identifier);

            match self.debug_channel.1.recv_deadline(Instant::now() + DEBUG_CHANNEL_DEADLINE) {
                Ok(id) => {
                    if id == *document_identifier {
                        debug!("successfully removed document id {}", document_identifier);
                        true
                    } else {
                        error!("expected document id {}, got {}", document_identifier, id);
                        false
                    }
                }
                Err(e) => match e {
                    RecvTimeoutError::Timeout => {
                        error!(
                            "did not received confirmation of buffer id {} being dropped - there is somewhere a live reference",
                            document_identifier
                        );
                        false
                    }
                    RecvTimeoutError::Disconnected => {
                        error!("debug channel broken");
                        false
                    }
                },
            }
        } else {
            error!("failed dropping document id {} - not found in registry", document_identifier);
            false
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
