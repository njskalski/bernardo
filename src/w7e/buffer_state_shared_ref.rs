use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockResult};

use log::error;

use crate::gladius::providers::Providers;
use crate::text::buffer_state::BufferState;
use crate::text::text_buffer::TextBuffer;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;

#[derive(Clone, Debug)]
pub struct BufferSharedRef(Arc<RwLock<BufferState>>);

impl BufferSharedRef {
    pub fn new_empty(tree_sitter_op: Option<Arc<TreeSitterWrapper>>) -> BufferSharedRef {
        let buffer_state = BufferState::full(tree_sitter_op);
        BufferSharedRef(Arc::new(RwLock::new(buffer_state)))
    }

    pub fn new_from_buffer(tree_sitter_op: Option<Arc<TreeSitterWrapper>>, buffer_state: BufferState) -> BufferSharedRef {
        BufferSharedRef(Arc::new(RwLock::new(buffer_state)))
    }

    pub fn lock(&self) -> Option<RwLockReadGuard<BufferState>> {
        match self.0.try_read() {
            Ok(lock) => {
                Some(lock)
            }
            Err(e) => {
                error!("failed to lock buffer for read!");
                None
            }
        }
    }

    pub fn lock_rw(&self) -> Option<RwLockWriteGuard<BufferState>> {
        match self.0.try_write() {
            Ok(lock) => Some(lock),
            Err(e) => {
                error!("failed to lock buffer for write!");
                None
            }
        }
    }
}

impl PartialEq<Self> for BufferSharedRef {
    fn eq(&self, other: &Self) -> bool {
        let my_ptr = Arc::as_ptr(&self.0);
        let other_ptr = Arc::as_ptr(&other.0);

        my_ptr == other_ptr
    }
}

impl Eq for BufferSharedRef {}

pub type BufferR<'a> = RwLockReadGuard<'a, BufferState>;
pub type BufferRW<'a> = RwLockWriteGuard<'a, BufferState>;
