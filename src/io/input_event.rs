use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::experiments::focus_group::FocusUpdate;
use crate::io::keys::Key;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InputEvent {
    Tick,
    // state might have changed, update and redraw if necessary
    KeyInput(Key),
    FocusUpdate(FocusUpdate),
    // primary feature - everything bar with escalation
    EverythingBarTrigger,
}

impl InputEvent {
    pub fn as_key(&self) -> Option<&Key> {
        match self {
            InputEvent::KeyInput(key) => Some(key),
            _ => None,
        }
    }
}
