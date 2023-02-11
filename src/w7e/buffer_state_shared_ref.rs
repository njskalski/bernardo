use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockResult};

use log::error;

use crate::gladius::providers::Providers;
use crate::text::buffer_state::BufferState;
use crate::text::text_buffer::TextBuffer;
use crate::tsw::tree_sitter_wrapper::TreeSitterWrapper;

pub struct BufferSharedRef(Arc<RwLock<BufferState>>);

impl BufferSharedRef {
    pub fn new_empty(tree_sitter_op: Option<Arc<TreeSitterWrapper>>) -> BufferSharedRef {
        let bf = BufferState::full(tree_sitter_op);

        BufferSharedRef(Arc::new(RwLock::new(bf)))
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

pub type BufferR<'a> = RwLockReadGuard<'a, BufferState>;
pub type BufferRW<'a> = RwLockWriteGuard<'a, BufferState>;
