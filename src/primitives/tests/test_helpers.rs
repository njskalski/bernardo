use crate::cursor::cursor_set::CursorSet;
use crate::cursor::tests::test_helpers::{assert_cursors_are_within_text, decode_text_and_cursors, encode_cursors_and_text};
use crate::experiments::clipboard::ClipboardRef;
use crate::primitives::common_edit_msgs::{apply_common_edit_message, CommonEditMsg};
use crate::primitives::has_invariant::HasInvariant;

pub fn decode_apply_and_encode_back(text: &str, cem: CommonEditMsg, clipboard: Option<&ClipboardRef>) -> String {
    let (mut buffer, mut cs) = decode_text_and_cursors(text);
    debug_assert!(cs.check_invariant());
    assert_cursors_are_within_text(&buffer, &cs);
    apply_common_edit_message(cem, &mut cs, &mut vec![], &mut buffer, 4, clipboard);
    debug_assert!(cs.check_invariant());
    assert_cursors_are_within_text(&buffer, &cs);
    encode_cursors_and_text(&buffer, &cs)
}

/*
This converts "set of cursors over same buffer", and cem, and apply cem to "selected"
**cursor set**. The reason we read "multiple texts" is that we can encode different cursor sets in
each of them, so first string encodes first set, second string second set etc.

Without cursors, the texts are expected to be identical, but I don't remember whether it's ever
checked.

It's created to facilitate test of "we have multiple views of the same buffer, one makes changes,
rest is updated accordingly".
 */
pub fn decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(
    texts: &Vec<&str>,
    selected: usize,
    cem: CommonEditMsg,
    clipboard: Option<&ClipboardRef>,
) -> Vec<String> {
    assert!(texts.len() > 1);
    assert!(selected < texts.len());

    let mut buffer_cs_pair = texts.iter().map(|text| decode_text_and_cursors(text)).collect::<Vec<_>>();

    for i in 1..buffer_cs_pair.len() {
        assert_eq!(buffer_cs_pair[0].0, buffer_cs_pair[i].0)
    }

    for it in buffer_cs_pair.iter() {
        assert!(it.1.check_invariant());
        assert_cursors_are_within_text(&it.0, &it.1);
    }

    let mut other_cursors: Vec<&mut CursorSet> = Vec::new();
    let mut buffer = buffer_cs_pair[0].0.clone();

    let mut selected_cursor_set: Option<&mut CursorSet> = None;

    for (idx, (_rope, cursors)) in buffer_cs_pair.iter_mut().enumerate() {
        if idx == selected {
            selected_cursor_set = Some(cursors);
        } else {
            other_cursors.push(cursors)
        }
    }

    apply_common_edit_message(cem, selected_cursor_set.unwrap(), &mut other_cursors, &mut buffer, 4, clipboard);

    let mut results: Vec<String> = Vec::new();

    for it in buffer_cs_pair.iter() {
        assert!(it.1.check_invariant());
        assert_cursors_are_within_text(&buffer, &it.1);

        let s = encode_cursors_and_text(&buffer, &it.1);
        results.push(s);
    }

    results
}
