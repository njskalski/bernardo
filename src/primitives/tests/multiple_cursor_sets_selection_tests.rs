#![cfg_attr(rustfmt, rustfmt_skip)]

/*
Here are tests of a quite exotic scenario, which Gladius aims at supporting: imagine we have 
multiple views of the same buffer with different cursor sets. We modify buffer in one view, so 
"common edit message" is applied to one set of cursors, and others are updated so they remain valid.

This update is potentially destructive (no way to keep a cursor over deleted part etc.).

This cannot be done perfectly, but we can do our best.

Each of the strings in argument   
 */

use crate::experiments::clipboard::Clipboard;
use crate::mocks::mock_clipboard::MockClipboard;
use crate::primitives::common_edit_msgs::CommonEditMsg;
use crate::primitives::tests::test_helpers::decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back;

#[test]
fn multiple_cursor_test_1_1() {
    let texts: Vec<&str> = vec![
        "fir.stte#st",
        "fir#stte.st",
    ];

    // Backspace
    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 0, CommonEditMsg::Backspace, None);

    assert_eq!(new_texts[0].as_str(), "firstt#st");
    assert_eq!(new_texts[1].as_str(), "fir#sttst");
}


#[test]
fn multiple_cursor_test_1_2() {
    let texts: Vec<&str> = vec![
        "fir.stte#st",
        "fir#stte.st",
    ];

    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 1, CommonEditMsg::Backspace, None);

    assert_eq!(new_texts[0].as_str(), "fistte#st");
    assert_eq!(new_texts[1].as_str(), "fi#sttest");
}

#[test]
fn multiple_cursor_test_2_1() {
    let texts: Vec<&str> = vec![
        "fir.stte#st",
        "fir#stte.st",
    ];
    // Delete
    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 0, CommonEditMsg::Delete, None);

    assert_eq!(new_texts[0].as_str(), "firstte#t");
    assert_eq!(new_texts[1].as_str(), "fir#sttet");
}

#[test]
fn multiple_cursor_test_2_2() {
    let texts: Vec<&str> = vec![
        "fir.stte#st",
        "fir#stte.st",
    ];

    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 1, CommonEditMsg::Delete, None);

    assert_eq!(new_texts[0].as_str(), "firtte#st");
    assert_eq!(new_texts[1].as_str(), "fir#ttest");
}

#[test]
fn multiple_cursor_test_3_1() {
    let texts: Vec<&str> = vec![
        "fir[st.te)s.t",
        "fir.st.te.s#t",
        "fir.st#te#s.t",
    ];


    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 0, CommonEditMsg::Backspace, None);

    assert_eq!(new_texts[0].as_str(), "fir#st");
    assert_eq!(new_texts[1].as_str(), "firs#t");
    assert_eq!(new_texts[2].as_str(), "fir#st");
}

#[test]
fn multiple_cursor_test_3_2() {
    let texts: Vec<&str> = vec![
        "fir[st.te)s.t",
        "fir.st.te.s#t",
        "fir.st#te#s.t",
    ];


    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 1, CommonEditMsg::Backspace, None);

    assert_eq!(new_texts[0].as_str(), "fir[stte)t");
    assert_eq!(new_texts[1].as_str(), "firstte#t");
    assert_eq!(new_texts[2].as_str(), "first#te#t");
}

#[test]
fn multiple_cursor_test_3_3() {
    let texts: Vec<&str> = vec![
        "fir[st.te)s.t",
        "fir.st.te.s#t",
        "fir.st#te#s.t",
    ];


    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 2, CommonEditMsg::Backspace, None);

    assert_eq!(new_texts[0].as_str(), "fir[st)st");
    assert_eq!(new_texts[1].as_str(), "firsts#t");
    assert_eq!(new_texts[2].as_str(), "firs#t#st");
}

#[test]
fn multiple_cursor_test_3_4() {
    let texts: Vec<&str> = vec![
        "fir[st.te)s.t",
        "fir.st.te.s#t",
        "fir.st#te#s.t",
    ];


    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 0, CommonEditMsg::Delete, None);

    assert_eq!(new_texts[0].as_str(), "fir#st");
    assert_eq!(new_texts[1].as_str(), "firs#t");
    assert_eq!(new_texts[2].as_str(), "fir#st");
}

#[test]
fn multiple_cursor_test_3_5() {
    let texts: Vec<&str> = vec![
        "fir[st.te)s.t",
        "fir.st.te.s#t",
        "fir.st#te#s.t",
    ];


    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 1, CommonEditMsg::Delete, None);

    assert_eq!(new_texts[0].as_str(), "fir[stte)s");
    assert_eq!(new_texts[1].as_str(), "firsttes#");
    assert_eq!(new_texts[2].as_str(), "first#te#s");
}

#[test]
fn multiple_cursor_test_3_6() {
    let texts: Vec<&str> = vec![
        "fir[st.te)s.t",
        "fir.st.te.s#t",
        "fir.st#te#s.t",
    ];


    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 2, CommonEditMsg::Delete, None);

    assert_eq!(new_texts[0].as_str(), "fir[ste)t");
    assert_eq!(new_texts[1].as_str(), "firste#t");
    assert_eq!(new_texts[2].as_str(), "first#e#t");
}

#[test]
fn multiple_cursor_test_4_1() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 0, CommonEditMsg::Backspace, None);

    assert_eq!(new_texts[0].as_str(), "fir#st");
    assert_eq!(new_texts[1].as_str(), "fir#st");
}

#[test]
fn multiple_cursor_test_4_2() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 1, CommonEditMsg::Backspace, None);

    assert_eq!(new_texts[0].as_str(), "fi[stt)st");
    assert_eq!(new_texts[1].as_str(), "fi#stt#st");
}

#[test]
fn multiple_cursor_test_4_3() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 0, CommonEditMsg::Delete, None);

    assert_eq!(new_texts[0].as_str(), "fir#st");
    assert_eq!(new_texts[1].as_str(), "fir#st");
}

#[test]
fn multiple_cursor_test_4_4() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 1, CommonEditMsg::Delete, None);

    assert_eq!(new_texts[0].as_str(), "fir[tte)t");
    assert_eq!(new_texts[1].as_str(), "fir#tte#t");
}


#[test]
fn multiple_cursor_test_5_1() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 0, CommonEditMsg::Char('a'), None);

    assert_eq!(new_texts[0].as_str(), "fira#st");
    // this I don't know, the intuition was fir#a#st
    assert_eq!(new_texts[1].as_str(), "fira#st");
}

#[test]
fn multiple_cursor_test_5_2() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 1, CommonEditMsg::Char('a'), None);

    assert_eq!(new_texts[0].as_str(), "fira[stte)ast");
    assert_eq!(new_texts[1].as_str(), "fira#sttea#st");
}

#[test]
fn multiple_cursor_test_6_1() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let clipboard = MockClipboard::default();
    clipboard.set("xxx".to_string());

    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 0, CommonEditMsg::Paste, Some(&clipboard.into_clipboardref()));

    assert_eq!(new_texts[0].as_str(), "firxxx#st");
    assert_eq!(new_texts[1].as_str(), "firxxx#st");
}

#[test]
fn multiple_cursor_test_6_2() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let clipboard = MockClipboard::default();
    clipboard.set("xxx".to_string());

    let new_texts = decode_multiple_sets_and_apply_at_selected_cursor_set_and_encode_back(&texts, 1, CommonEditMsg::Paste, Some(&clipboard.into_clipboardref()));

    assert_eq!(new_texts[0].as_str(), "firxxx[stte)xxxst");
    assert_eq!(new_texts[1].as_str(), "firxxx#sttexxx#st");
}
