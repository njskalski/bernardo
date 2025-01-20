use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::text::buffer_state::BufferState;
use crate::text::buffer_type::BufferType;
use crate::text::contents_and_cursors::ContentsAndCursors;
use crate::text::document_identifier::DocumentIdentifier;
use crate::text::lang_id::LangId;
use crate::text::tree_sitter_wrapper::TreeSitterWrapper;
use crate::experiments::clipboard::ClipboardRef;
use crate::io::keys::WID;
use std::sync::{Arc, mpsc};

#[test]
fn test_cursor_movement_does_not_modify_buffer() {
    // Initialize a buffer state with some text
    let initial_text = "Hello, world!";
    let mut buffer_state = BufferState {
        subtype: BufferType::MultiLine,
        tree_sitter_op: None,
        history: vec![ContentsAndCursors::new(initial_text.to_string())],
        history_pos: 0,
        last_save_pos: None,
        lang_id: None,
        document_identifier: DocumentIdentifier::new(),
        drop_notice_sink: None,
    };

    // Define cursor movement messages
    let cursor_movements = vec![
        CommonEditMsg::CursorUp { selecting: false },
        CommonEditMsg::CursorDown { selecting: false },
        CommonEditMsg::CursorLeft { selecting: false },
        CommonEditMsg::CursorRight { selecting: false },
        CommonEditMsg::LineBegin { selecting: false },
        CommonEditMsg::LineEnd { selecting: false },
        CommonEditMsg::WordBegin { selecting: false },
        CommonEditMsg::WordEnd { selecting: false },
        CommonEditMsg::PageUp { selecting: false },
        CommonEditMsg::PageDown { selecting: false },
    ];

    // Apply each cursor movement message and check that the buffer is not modified
    for msg in cursor_movements {
        // Clone the buffer state to compare after applying the message
        let buffer_state_clone = buffer_state.clone();

        // Apply the cursor movement message
        buffer_state.apply_common_edit_message(msg, WID::new(), 1, None, false);

        // Assert that the buffer text is unchanged
        assert_eq!(
            buffer_state.history[buffer_state.history_pos].text(),
            buffer_state_clone.history[buffer_state_clone.history_pos].text(),
            "Buffer was modified by cursor movement: {:?}",
            msg
        );
    }
}