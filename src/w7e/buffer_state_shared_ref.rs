use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use log::error;

use crate::text::buffer_state::BufferState;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;
use crate::widgets::main_view::main_view::DocumentIdentifier;

#[derive(Clone, Debug)]
pub struct BufferSharedRef {
    buffer: Arc<RwLock<BufferState>>,
    identifier: DocumentIdentifier,
}

impl BufferSharedRef {
    pub fn new_empty(tree_sitter_op: Option<Arc<TreeSitterWrapper>>, tabs_to_spaces: Option<u8>) -> BufferSharedRef {
        let id = DocumentIdentifier::new_unique();
        let buffer_state = BufferState::full(tree_sitter_op, id.clone(), None, tabs_to_spaces);
        BufferSharedRef {
            buffer: Arc::new(RwLock::new(buffer_state)),
            identifier: id,
        }
    }

    pub fn new_from_buffer(buffer_state: BufferState) -> BufferSharedRef {
        let id = buffer_state.get_document_identifier().clone();

        BufferSharedRef {
            buffer: Arc::new(RwLock::new(buffer_state)),
            identifier: id,
        }
    }

    pub fn document_identifier(&self) -> &DocumentIdentifier {
        &self.identifier
    }

    pub fn lock(&self) -> Option<RwLockReadGuard<BufferState>> {
        match self.buffer.try_read() {
            Ok(lock) => Some(lock),
            Err(e) => {
                error!("failed to lock buffer for read! : {}", e);
                None
            }
        }
    }

    pub fn lock_rw(&self) -> Option<RwLockWriteGuard<BufferState>> {
        match self.buffer.try_write() {
            Ok(lock) => Some(lock),
            Err(e) => {
                error!("failed to lock buffer for write! : {}", e);
                None
            }
        }
    }
}

impl PartialEq<Self> for BufferSharedRef {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for BufferSharedRef {}

pub type BufferR<'a> = RwLockReadGuard<'a, BufferState>;
pub type BufferRW<'a> = RwLockWriteGuard<'a, BufferState>;
