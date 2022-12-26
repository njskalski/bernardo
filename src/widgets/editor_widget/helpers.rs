use log::{debug, error};
use streaming_iterator::StreamingIterator;
use unicode_width::UnicodeWidthStr;

use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::primitives::cursor_set::{Cursor, CursorSet};
use crate::primitives::helpers::copy_last_n_columns;
use crate::primitives::xy::XY;
use crate::text::buffer_state::BufferState;
use crate::text::text_buffer::TextBuffer;
use crate::unpack_or;
use crate::w7e::navcomp_provider::StupidSubstituteMessage;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CursorScreenPosition {
    pub cursor: Cursor,
    pub screen_space: Option<XY>,
    pub absolute: XY,
}

//TODO tests
pub fn find_trigger_and_substring<'a>(triggers: &'a Vec<String>, buffer: &'a dyn TextBuffer, cursor_pos: &'a CursorScreenPosition) -> Option<(&'a String, String)> {
    let cursor_screen_pos = match cursor_pos.screen_space {
        None => {
            debug!("cursor not visible");
            return None;
        }
        Some(csp) => csp,
    };

    let how_many_columns_visible = cursor_screen_pos.x;
    let how_many_columns_total = cursor_pos.absolute.x;

    debug_assert!(how_many_columns_visible <= how_many_columns_total);
    if how_many_columns_visible == 0 {
        debug!("no columns visible");
        return None;
    }

    let entire_line = match buffer.lines().skip(cursor_pos.absolute.y as usize).next() {
        None => {
            error!("couldn't find line {} (drawn as +1) to harvest substring", cursor_pos.absolute.y);
            return None;
        }
        Some(line_contents) => line_contents.trim().to_string(),
    };

    debug!("read [{}] from begin of {} (drawn as +1) line", entire_line, cursor_pos.absolute.y);

    let cut_line = match copy_last_n_columns(&entire_line, how_many_columns_visible as usize, true) {
        None => {
            debug!("for some reason cutting last n columns failed");
            return None;
        }
        Some(l) => l,
    };

    let mut position_of_first_char_after_last_char_of_trigger_within_cut_line: Option<usize> = None;
    let mut selected_trigger: Option<&String> = None;
    for trigger in triggers {
        if let Some(pos) = cut_line.rfind(trigger) {
            position_of_first_char_after_last_char_of_trigger_within_cut_line = Some(pos + trigger.width());
            selected_trigger = Some(trigger);
            break;
        }
    }

    let substring = position_of_first_char_after_last_char_of_trigger_within_cut_line.map(|p| {
        cut_line[p..].to_string()
    });

    if substring.is_some() {
        Some((selected_trigger.unwrap(), substring.unwrap()))
    } else {
        None
    }
}
