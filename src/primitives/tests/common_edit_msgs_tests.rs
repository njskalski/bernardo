#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::cursor::tests::cursor_tests_common::{
    assert_cursors_are_within_text, decode_text_and_cursors, encode_cursors_and_text,
};
use crate::experiments::clipboard::{ClipboardRef, get_me_fake_clipboard};
use crate::primitives::common_edit_msgs::{apply_common_edit_message, CommonEditMsg};
use crate::primitives::has_invariant::HasInvariant;

fn decode_apply_and_encode_back(text: &str, cem: CommonEditMsg, clipboard: Option<&ClipboardRef>) -> String {
    let (mut buffer, mut cs) = decode_text_and_cursors(text);
    debug_assert!(cs.check_invariant());
    assert_cursors_are_within_text(&buffer, &cs);
    apply_common_edit_message(cem, &mut cs, &mut vec![], &mut buffer, 4, clipboard);
    debug_assert!(cs.check_invariant());
    assert_cursors_are_within_text(&buffer, &cs);
    encode_cursors_and_text(&buffer, &cs)
}

#[test]
fn single_cursor_write() {
    assert_eq!(decode_apply_and_encode_back("ab#ba", CommonEditMsg::Char('c'), None), "abc#ba");
    assert_eq!(decode_apply_and_encode_back("#abba", CommonEditMsg::Char('c'), None), "c#abba");
    assert_eq!(decode_apply_and_encode_back("abba#", CommonEditMsg::Char('c'), None), "abbac#");
}

#[test]
fn single_cursor_block_write() {
    assert_eq!(decode_apply_and_encode_back("ab#ba", CommonEditMsg::Block("hello".to_string()), None), "abhello#ba");
    assert_eq!(decode_apply_and_encode_back("#abba", CommonEditMsg::Block("hello".to_string()), None), "hello#abba");
}

#[test]
fn single_cursor_block_replace() {
    // assert_eq!(text_to_text("ab[ba)x", CommonEditMsg::Block("hello".to_string()), None),
    // "abhello#x");
    assert_eq!(
        decode_apply_and_encode_back("ab(ba]x", CommonEditMsg::Block("hello".to_string()), None),
        "abhello#x"
    );
}

#[test]
fn single_cursor_backspace() {
    assert_eq!(decode_apply_and_encode_back("ab#ba", CommonEditMsg::Backspace, None), "a#ba");
    assert_eq!(decode_apply_and_encode_back("#abba", CommonEditMsg::Backspace, None), "#abba");
    assert_eq!(decode_apply_and_encode_back("abba#", CommonEditMsg::Backspace, None), "abb#");
}

#[test]
fn single_cursor_delete() {
    assert_eq!(decode_apply_and_encode_back("ab#da", CommonEditMsg::Delete, None), "ab#a");
    assert_eq!(decode_apply_and_encode_back("abda#", CommonEditMsg::Delete, None), "abda#");
    assert_eq!(decode_apply_and_encode_back("#abda", CommonEditMsg::Delete, None), "#bda");
}

#[test]
fn multi_cursor_write() {
    assert_eq!(decode_apply_and_encode_back("abc#abc#a", CommonEditMsg::Char('d'), None), "abcd#abcd#a");
    assert_eq!(
        decode_apply_and_encode_back("abc#abc#a", CommonEditMsg::Block("hello".to_string()), None),
        "abchello#abchello#a"
    );
}

#[test]
fn multi_cursor_block_selection() {
    assert_eq!(
        decode_apply_and_encode_back("(ab]c(ab]c", CommonEditMsg::Block("hello".to_string()), None),
        "hello#chello#c"
    );
    assert_eq!(
        decode_apply_and_encode_back("[ab)c[ab)c", CommonEditMsg::Block("hello".to_string()), None),
        "hello#chello#c"
    );
}

#[test]
fn scenario_1() {
    assert_eq!(decode_apply_and_encode_back("#\n#\n#\n#\n", CommonEditMsg::Char('a'), None), "a#\na#\na#\na#\n");
    assert_eq!(
        decode_apply_and_encode_back("a#\na#\na#\na#\n", CommonEditMsg::Char('b'), None),
        "ab#\nab#\nab#\nab#\n"
    );
    assert_eq!(
        decode_apply_and_encode_back("ab#\nab#\nab#\nab#\n", CommonEditMsg::CursorLeft { selecting: true }, None),
        "a[b)\na[b)\na[b)\na[b)\n"
    );
    assert_eq!(
        decode_apply_and_encode_back("a[b)\na[b)\na[b)\na[b)\n", CommonEditMsg::Char('x'), None),
        "ax#\nax#\nax#\nax#\n"
    );
    assert_eq!(
        decode_apply_and_encode_back("ax#\nax#\nax#\nax#\n", CommonEditMsg::WordBegin { selecting: true }, None),
        "[ax)\n[ax)\n[ax)\n[ax)\n"
    );
    assert_eq!(
        decode_apply_and_encode_back("[ax)\n[ax)\n[ax)\n[ax)\n", CommonEditMsg::Char('u'), None),
        "u#\nu#\nu#\nu#\n"
    );
    assert_eq!(decode_apply_and_encode_back("u#\nu#\nu#\nu#\n", CommonEditMsg::Backspace, None), "#\n#\n#\n#\n");
    assert_eq!(decode_apply_and_encode_back("#\n#\n#\n#\n", CommonEditMsg::Backspace, None), "#\n");
}

#[test]
fn multi_cursor_backspace() {
    assert_eq!(decode_apply_and_encode_back("#\n#\n#\n#\n", CommonEditMsg::Backspace, None), "#\n");
}

#[test]
fn multi_cursor_delete() {
    assert_eq!(decode_apply_and_encode_back("#ab#ab#ab#ab", CommonEditMsg::Delete, None), "#b#b#b#b");
    assert_eq!(decode_apply_and_encode_back("#\n#\n#\n#\n", CommonEditMsg::Delete, None), "#");
}

#[test]
fn multi_cursor_copy_paste() {
    let clipboard = get_me_fake_clipboard();
    let c = Some(&clipboard);

    assert_eq!(
        decode_apply_and_encode_back("#abba\n#abba\n#abba\n#abba\n", CommonEditMsg::CursorRight { selecting: true }, c),
        "(a]bba\n(a]bba\n(a]bba\n(a]bba\n"
    );
    assert_eq!(
        decode_apply_and_encode_back(
            "(a]bba\n(a]bba\n(a]bba\n(a]bba\n",
            CommonEditMsg::CursorRight { selecting: true },
            c,
        ),
        "(ab]ba\n(ab]ba\n(ab]ba\n(ab]ba\n"
    );
    assert_eq!(
        decode_apply_and_encode_back("(ab]ba\n(ab]ba\n(ab]ba\n(ab]ba\n", CommonEditMsg::Copy, c),
        "(ab]ba\n(ab]ba\n(ab]ba\n(ab]ba\n"
    );
    assert_eq!(
        decode_apply_and_encode_back("(ab]ba\n(ab]ba\n(ab]ba\n(ab]ba\n", CommonEditMsg::LineEnd { selecting: false }, c),
        "abba#\nabba#\nabba#\nabba#\n"
    );
    assert_eq!(
        decode_apply_and_encode_back("abba#\nabba#\nabba#\nabba#\n", CommonEditMsg::Paste, c),
        "abbaab#\nabbaab#\nabbaab#\nabbaab#\n"
    );
}

#[test]
fn delete_block() {
    assert_eq!(
        decode_apply_and_encode_back("#alamakota#kot#", CommonEditMsg::DeleteBlock { char_range: 1..4 }, None),
        "#aakota#kot#"
    );
    assert_eq!(
        decode_apply_and_encode_back("alamakota[kot)", CommonEditMsg::DeleteBlock { char_range: 1..4 }, None),
        "aakota[kot)"
    );
}

#[test]
fn insert_block() {
    assert_eq!(
        decode_apply_and_encode_back(
            "#alamakota#kot#",
            CommonEditMsg::InsertBlock {
                char_pos: 0,
                what: "dupa".to_string(),
            },
            None,
        ),
        "dupa#alamakota#kot#"
    );
    assert_eq!(
        decode_apply_and_encode_back(
            "dupa[kot)",
            CommonEditMsg::InsertBlock {
                char_pos: 5,
                what: "nic".to_string(),
            },
            None,
        ),
        "dupa[knicot)"
    );
}

#[test]
fn shift_tab_1() {
    let text_1 = "
aa#aa
    bbbb#
      ccc#c";
    let text_1_after = "
aa#aa
bbbb#
  ccc#c";

    assert_eq!(decode_apply_and_encode_back(text_1, CommonEditMsg::ShiftTab, None), text_1_after);
}

#[test]
fn shift_tab_2() {
    let text_1 = "
somebs
[
aaaa
    bbbb
      ccc)c";
    let text_1_after = "
somebs
[
aaaa
bbbb
  ccc)c";

    assert_eq!(decode_apply_and_encode_back(text_1, CommonEditMsg::ShiftTab, None), text_1_after);
}

#[test]
fn tab_1() {
    let text_1 = "
aa#aa
    bbbb#
      ccc#c";
    let text_1_after = "
aa    #aa
    bbbb    #
      ccc    #c";

    assert_eq!(decode_apply_and_encode_back(text_1, CommonEditMsg::Tab, None), text_1_after);
}

#[test]
fn tab_2() {
    let text_1 = "
aa(aa
    bbbb
      ccc]c
      dddd";
    let text_1_after = "
    aa(aa
        bbbb
          ccc]c
      dddd";
    assert_eq!(decode_apply_and_encode_back(text_1, CommonEditMsg::Tab, None), text_1_after);
}
