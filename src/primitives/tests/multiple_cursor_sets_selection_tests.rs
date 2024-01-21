#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::cursor::cursor_set::CursorSet;
use crate::cursor::tests::cursor_tests_common::{common_assert_pair_makes_sense, common_buffer_cursors_sel_to_text, common_text_to_buffer_cursors_with_selections};
use crate::experiments::clipboard::{Clipboard, ClipboardRef};
use crate::mocks::mock_clipboard::MockClipboard;
use crate::primitives::common_edit_msgs::{_apply_cem, CommonEditMsg};
use crate::primitives::has_invariant::HasInvariant;

/*
This converts "set of cursors over same buffer", and cem, and apply cem to "selected" one, and
update the others accordingly.
 */
fn texts_to_texts(texts: &Vec<&str>, selected: usize, cem: CommonEditMsg, clipboard: Option<&ClipboardRef>) -> Vec<String> {
    assert!(texts.len() > 1);
    assert!(selected < texts.len());

    let mut buffer_cs_pair = texts.iter().map(|text| {
        common_text_to_buffer_cursors_with_selections(text)
    }).collect::<Vec<_>>();

    for i in 1..buffer_cs_pair.len() {
        assert_eq!(buffer_cs_pair[0].0, buffer_cs_pair[i].0)
    }

    for it in buffer_cs_pair.iter() {
        assert!(it.1.check_invariant());
        common_assert_pair_makes_sense(&it.0, &it.1);
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

    _apply_cem(cem, selected_cursor_set.unwrap(), &mut other_cursors, &mut buffer, 4, clipboard);

    let mut results: Vec<String> = Vec::new();

    for it in buffer_cs_pair.iter() {
        assert!(it.1.check_invariant());
        common_assert_pair_makes_sense(&buffer, &it.1);

        let s = common_buffer_cursors_sel_to_text(&buffer, &it.1);
        results.push(s);
    }

    results
}

#[test]
fn multiple_cursor_test_1_1() {
    let texts: Vec<&str> = vec![
        "fir.stte#st",
        "fir#stte.st",
    ];

    // Backspace
    let new_texts = texts_to_texts(&texts, 0, CommonEditMsg::Backspace, None);

    assert_eq!(new_texts[0].as_str(), "firstt#st");
    assert_eq!(new_texts[1].as_str(), "fir#sttst");
}

#[test]
fn multiple_cursor_test_1_2() {
    let texts: Vec<&str> = vec![
        "fir.stte#st",
        "fir#stte.st",
    ];

    let new_texts = texts_to_texts(&texts, 1, CommonEditMsg::Backspace, None);

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
    let new_texts = texts_to_texts(&texts, 0, CommonEditMsg::Delete, None);

    assert_eq!(new_texts[0].as_str(), "firstte#t");
    assert_eq!(new_texts[1].as_str(), "fir#sttet");
}

#[test]
fn multiple_cursor_test_2_2() {
    let texts: Vec<&str> = vec![
        "fir.stte#st",
        "fir#stte.st",
    ];

    let new_texts = texts_to_texts(&texts, 1, CommonEditMsg::Delete, None);

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


    let new_texts = texts_to_texts(&texts, 0, CommonEditMsg::Backspace, None);

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


    let new_texts = texts_to_texts(&texts, 1, CommonEditMsg::Backspace, None);

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


    let new_texts = texts_to_texts(&texts, 2, CommonEditMsg::Backspace, None);

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


    let new_texts = texts_to_texts(&texts, 0, CommonEditMsg::Delete, None);

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


    let new_texts = texts_to_texts(&texts, 1, CommonEditMsg::Delete, None);

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


    let new_texts = texts_to_texts(&texts, 2, CommonEditMsg::Delete, None);

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

    let new_texts = texts_to_texts(&texts, 0, CommonEditMsg::Backspace, None);

    assert_eq!(new_texts[0].as_str(), "fir#st");
    assert_eq!(new_texts[1].as_str(), "fir#st");
}

#[test]
fn multiple_cursor_test_4_2() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let new_texts = texts_to_texts(&texts, 1, CommonEditMsg::Backspace, None);

    assert_eq!(new_texts[0].as_str(), "fi[stt)st");
    assert_eq!(new_texts[1].as_str(), "fi#stt#st");
}

#[test]
fn multiple_cursor_test_4_3() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let new_texts = texts_to_texts(&texts, 0, CommonEditMsg::Delete, None);

    assert_eq!(new_texts[0].as_str(), "fir#st");
    assert_eq!(new_texts[1].as_str(), "fir#st");
}

#[test]
fn multiple_cursor_test_4_4() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let new_texts = texts_to_texts(&texts, 1, CommonEditMsg::Delete, None);

    assert_eq!(new_texts[0].as_str(), "fir[tte)t");
    assert_eq!(new_texts[1].as_str(), "fir#tte#t");
}


#[test]
fn multiple_cursor_test_5_1() {
    let texts: Vec<&str> = vec![
        "fir[stte)st",
        "fir#stte#st",
    ];

    let new_texts = texts_to_texts(&texts, 0, CommonEditMsg::Char('a'), None);

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

    let new_texts = texts_to_texts(&texts, 1, CommonEditMsg::Char('a'), None);

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

    let new_texts = texts_to_texts(&texts, 0, CommonEditMsg::Paste, Some(&clipboard.into_clipboardref()));

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

    let new_texts = texts_to_texts(&texts, 1, CommonEditMsg::Paste, Some(&clipboard.into_clipboardref()));

    assert_eq!(new_texts[0].as_str(), "firxxx[stte)xxxst");
    assert_eq!(new_texts[1].as_str(), "firxxx#sttexxx#st");
}
