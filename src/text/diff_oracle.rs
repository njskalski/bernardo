/*
This decides whether a particular CommonEditMessage justifies new milestone
 */

use std::time::{Duration, SystemTime};
use crate::experiments::clipboard::ClipboardRef;
use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::primitives::cursor_set::CursorSet;

const MIN_CHARS: usize = 10;
const MIN_DURATION: Duration = Duration::from_secs(3);

pub struct DiffOracle {
    last_timestamp: SystemTime,
    total_change_chars: usize,
}

impl Default for DiffOracle {
    fn default() -> Self {
        DiffOracle {
            last_timestamp: SystemTime::now(),
            total_change_chars: 0,
        }
    }
}

impl DiffOracle {
    fn internal_trigger(&mut self) -> bool {
        let mut trigger = false;

        trigger |= self.total_change_chars >= MIN_CHARS;
        let now = SystemTime::now();

        if let Ok(delta) = now.duration_since(self.last_timestamp) {
            trigger |= delta >= MIN_DURATION;
        }

        if trigger {
            self.total_change_chars = 0;
            self.last_timestamp = self.last_timestamp.max(now);
        }

        trigger
    }

    /*
    returns, whether the new milestone should be created or not
     */
    pub fn update_with(&mut self, cem: &CommonEditMsg, cs: &CursorSet, clipboard: Option<&ClipboardRef>) -> bool {
        match cem {
            CommonEditMsg::Char(_) => {
                self.total_change_chars += cs.set().len();
            }
            CommonEditMsg::CursorUp { .. } => {}
            CommonEditMsg::CursorDown { .. } => {}
            CommonEditMsg::CursorLeft { .. } => {}
            CommonEditMsg::CursorRight { .. } => {}
            CommonEditMsg::Backspace => {}
            CommonEditMsg::LineBegin { .. } => {}
            CommonEditMsg::LineEnd { .. } => {}
            CommonEditMsg::WordBegin { .. } => {}
            CommonEditMsg::WordEnd { .. } => {}
            CommonEditMsg::PageUp { .. } => {}
            CommonEditMsg::PageDown { .. } => {}
            CommonEditMsg::Delete => {}
            CommonEditMsg::Copy => {}
            CommonEditMsg::Paste => {}
            CommonEditMsg::Undo => {}
            CommonEditMsg::Redo => {}
        }
    }
}