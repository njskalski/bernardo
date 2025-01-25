use crate::cursor::tests::test_helpers::{decode_text_and_cursors, encode_cursors_and_text};
use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::text::buffer_state::BufferState;
use crate::text::text_buffer::TextBuffer;
use crate::widget::widget::{get_new_widget_id, WID};
use crate::widgets::main_view::main_view::DocumentIdentifier;

fn encode_buffer_state(text: &str) -> (BufferState, WID) {
    let (text, cursors) = decode_text_and_cursors(text);
    let wid: WID = get_new_widget_id();

    let mut bs = BufferState::full(None, DocumentIdentifier::new_unique(), None, None).with_text(text.to_string());
    assert!(bs.text_mut().add_cursor_set(wid, cursors));

    (bs, wid)
}

fn decode_buffer_state(buffer_state: &BufferState, wid: WID) -> String {
    let text = buffer_state.text();
    encode_cursors_and_text(text.rope(), text.get_cursor_set(wid).unwrap())
}

// this is not a great test, but it got broken once already, so it's really "keeping the lights on".
#[test]
fn test_undo_redo_1() {
    let (mut buffer_state, wid) = encode_buffer_state("ala ma kota#");

    for _ in 0..4 {
        assert!(
            buffer_state
                .apply_common_edit_message(CommonEditMsg::Backspace, wid, 4, None, true)
                .modified_buffer
        );
    }

    assert_eq!(decode_buffer_state(&buffer_state, wid), "ala ma #".to_string());

    assert!(buffer_state.undo());
    assert_eq!(decode_buffer_state(&buffer_state, wid), "ala ma k#".to_string());
    assert!(buffer_state.undo());
    assert_eq!(decode_buffer_state(&buffer_state, wid), "ala ma ko#".to_string());
    assert!(buffer_state.undo());
    assert_eq!(decode_buffer_state(&buffer_state, wid), "ala ma kot#".to_string());
    assert!(buffer_state.undo());
    assert_eq!(decode_buffer_state(&buffer_state, wid), "ala ma kota#".to_string());
    assert_eq!(buffer_state.undo(), false);

    assert!(buffer_state.can_redo());
    assert!(buffer_state.redo());
    assert_eq!(decode_buffer_state(&buffer_state, wid), "ala ma kot#".to_string());
    assert!(buffer_state.redo());
    assert_eq!(decode_buffer_state(&buffer_state, wid), "ala ma ko#".to_string());
    assert!(buffer_state.redo());
    assert_eq!(decode_buffer_state(&buffer_state, wid), "ala ma k#".to_string());
    assert!(buffer_state.redo());
    assert_eq!(decode_buffer_state(&buffer_state, wid), "ala ma #".to_string());
    assert_eq!(buffer_state.redo(), false);
}
