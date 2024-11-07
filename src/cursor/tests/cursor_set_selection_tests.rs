use ropey::Rope;

use crate::cursor::cursor::Cursor;
use crate::cursor::cursor::Selection;
use crate::cursor::cursor_set::CursorSet;
use crate::cursor::tests::test_helpers::{decode_text_and_cursors, encode_cursors_and_text};
use crate::text::text_buffer::TextBuffer;

pub fn apply_on_encoded_text_and_cursors(input: &str, f: fn(&mut CursorSet, &dyn TextBuffer) -> ()) -> String {
    let (bs, mut cs) = decode_text_and_cursors(input);
    f(&mut cs, &bs);
    encode_cursors_and_text(&bs, &cs)
}

#[test]
fn test_decode_text_and_cursors_1() {
    let (text, cursors) = decode_text_and_cursors("te[xt)");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 1);
    assert_eq!(cursors.set()[0].a, 2);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(2, 4)));
}

#[test]
fn test_decode_text_and_cursors_2() {
    let (text, cursors) = decode_text_and_cursors("te(xt]");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 1);
    assert_eq!(cursors.set()[0].a, 4);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(2, 4)));
}

#[test]
fn test_decode_text_and_cursors_3() {
    let (text, cursors) = decode_text_and_cursors("(t]e(xt]");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 2);
    assert_eq!(cursors.set()[0].a, 1);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(0, 1)));
    assert_eq!(cursors.set()[1].a, 4);
    assert_eq!(cursors.set()[1].s, Some(Selection::new(2, 4)));
}

#[test]
fn test_decode_text_and_cursors_4() {
    let (text, cursors) = decode_text_and_cursors("(te](xt]");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 2);
    assert_eq!(cursors.set()[0].a, 2);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(0, 2)));
    assert_eq!(cursors.set()[1].a, 4);
    assert_eq!(cursors.set()[1].s, Some(Selection::new(2, 4)));
}

#[test]
fn test_encode_cursors_and_text_0() {
    let text = encode_cursors_and_text(&Rope::from("text"), &CursorSet::new(vec![]));

    assert_eq!(text, "text");
}

#[test]
fn test_encode_cursors_and_text_1() {
    let text = encode_cursors_and_text(
        &Rope::from("text"),
        &CursorSet::new(vec![Cursor::new(0).with_selection(Selection::new(0, 2))]),
    );

    assert_eq!(text, "[te)xt");
}

#[test]
fn test_encode_cursors_and_text_2() {
    let text = encode_cursors_and_text(
        &Rope::from("text"),
        &CursorSet::new(vec![
            Cursor::new(0).with_selection(Selection::new(0, 2)),
            Cursor::new(2).with_selection(Selection::new(2, 4)),
        ]),
    );

    assert_eq!(text, "[te)[xt)");
}

#[test]
fn apply_sel_works() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |_c: &mut CursorSet, _b: &dyn TextBuffer| {};

    assert_eq!(apply_on_encoded_text_and_cursors("text", f), "text");
    assert_eq!(apply_on_encoded_text_and_cursors("te[xt)", f), "te[xt)");
    assert_eq!(apply_on_encoded_text_and_cursors("[t)(ext]", f), "[t)(ext]");
}

// these are actual tests of CursorSet with the selection.

#[test]
fn walking_over_selection_begin() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
        c.move_right(b, true);
    };

    let expected_process = vec![
        "m[issi)sipi",
        "mi[ssi)sipi",
        "mis[si)sipi",
        "miss[i)sipi",
        "missi#sipi",
        "missi(s]ipi",
        "missi(si]pi",
        "missi(sip]i",
        "missi(sipi]",
        "missi(sipi]",
        "missi(sipi]",
    ];

    for i in 0..expected_process.len() - 1 {
        assert_eq!(apply_on_encoded_text_and_cursors(expected_process[i], f), expected_process[i + 1]);
    }
}

#[test]
fn home() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
        c.move_home(b, true);
    };

    // assert_eq!(apply_sel("text", f), "text");
    assert_eq!(apply_on_encoded_text_and_cursors("text#", f), "[text)");
    assert_eq!(apply_on_encoded_text_and_cursors("te#xt", f), "[te)xt");
    assert_eq!(
        apply_on_encoded_text_and_cursors("a#aa\nbb#b\nccc#\n#", f),
        "[a)aa\n[bb)b\n[ccc)\n#"
    );
    assert_eq!(
        apply_on_encoded_text_and_cursors("a#aa\nb#b#b\nccc#\n#", f),
        "[a)aa\n[bb)b\n[ccc)\n#"
    );
    assert_eq!(
        apply_on_encoded_text_and_cursors("a#aa\nb#b#b\n#ccc#\n##", f),
        "[a)aa\n[bb)b\n[ccc)\n#"
    );
}

#[test]
fn end() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
        c.move_end(b, true);
    };

    // assert_eq!(apply_sel("text", f), "text");
    assert_eq!(apply_on_encoded_text_and_cursors("text#", f), "text#");
    assert_eq!(apply_on_encoded_text_and_cursors("te#xt", f), "te(xt]");
    assert_eq!(apply_on_encoded_text_and_cursors("a#aa\nbb#b\nccc#\n#", f), "a(aa]\nbb(b]\nccc#\n#");
    assert_eq!(
        apply_on_encoded_text_and_cursors("a#aa\nb#b#b\nccc#\n#", f),
        "a(aa]\nb(bb]\nccc#\n#"
    );
    assert_eq!(
        apply_on_encoded_text_and_cursors("a#aa\nb#b#b\n#ccc#\n##", f),
        "a(aa]\nb(bb]\n(ccc]\n#"
    );
}

#[test]
fn arrow_up_1() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
        c.move_vertically_by(b, -1, true);
    };

    // assert_eq!(apply_sel("text", f), "text");
    assert_eq!(apply_on_encoded_text_and_cursors("text#", f), "[text)");
    assert_eq!(apply_on_encoded_text_and_cursors("te#xt", f), "[te)xt");

    assert_eq!(
        apply_on_encoded_text_and_cursors("lin#e1\nline2\nli#ne3", f),
        "[lin)e1\nli[ne2\nli)ne3"
    );
    assert_eq!(
        apply_on_encoded_text_and_cursors("lin#e1\nline2\nli#ne3\n#", f),
        "[lin)e1\nli[ne2\n)[line3\n)"
    );
}

#[test]
fn arrow_up_2() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
        c.move_vertically_by(b, -1, true);
    };

    assert_eq!(
        apply_on_encoded_text_and_cursors("line1\nline2\nli#ne3", f),
        "line1\nli[ne2\nli)ne3"
    );
    assert_eq!(
        apply_on_encoded_text_and_cursors("line1\nli[ne2\nli)ne3", f),
        "li[ne1\nline2\nli)ne3"
    );
    assert_eq!(
        apply_on_encoded_text_and_cursors("li[ne1\nline2\nli)ne3", f),
        "[line1\nline2\nli)ne3"
    );
}

#[test]
fn arrow_down_1() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
        c.move_vertically_by(b, 1, true);
    };

    // assert_eq!(apply_sel("text", f), "text");
    assert_eq!(apply_on_encoded_text_and_cursors("#text", f), "(text]");
    assert_eq!(apply_on_encoded_text_and_cursors("te#xt", f), "te(xt]");

    assert_eq!(
        apply_on_encoded_text_and_cursors("lin#e1\nline2\nli#ne3", f),
        "lin(e1\nlin]e2\nli(ne3]"
    );
    assert_eq!(
        apply_on_encoded_text_and_cursors("lin#e1\nline2\nli#ne3\n#", f),
        "lin(e1\nlin]e2\nli(ne3\n]"
    );
}

#[test]
fn arrow_down_2() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
        c.move_vertically_by(b, 1, true);
    };

    assert_eq!(
        apply_on_encoded_text_and_cursors("li#ne1\nline2\nline3", f),
        "li(ne1\nli]ne2\nline3"
    );
    assert_eq!(
        apply_on_encoded_text_and_cursors("li(ne1\nli]ne2\nline3", f),
        "li(ne1\nline2\nli]ne3"
    );
    assert_eq!(
        apply_on_encoded_text_and_cursors("li(ne1\nline2\nli]ne3", f),
        "li(ne1\nline2\nline3]"
    );
}

#[test]
fn single_cursor_word_begin_with_selection() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, bs: &dyn TextBuffer| {
        c.word_begin_default(bs, true);
    };

    let progress = vec![
        "ala ma ko#ta",
        "ala ma [ko)ta",
        "ala ma[ ko)ta",
        "ala [ma ko)ta",
        "ala[ ma ko)ta",
        "[ala ma ko)ta",
        "[ala ma ko)ta",
    ];

    for i in 0..progress.len() - 1 {
        assert_eq!(apply_on_encoded_text_and_cursors(progress[i], f), progress[i + 1], "i: {}", i);
    }
}

#[test]
fn single_cursor_word_end_with_selection() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, bs: &dyn TextBuffer| {
        c.word_end_default(bs, true);
    };

    let progress = vec![
        "al#a ma kota",
        "al(a] ma kota",
        "al(a ]ma kota",
        "al(a ma] kota",
        "al(a ma ]kota",
        "al(a ma kota]",
        "al(a ma kota]",
    ];

    for i in 0..progress.len() - 1 {
        assert_eq!(apply_on_encoded_text_and_cursors(progress[i], f), progress[i + 1], "i: {}", i);
    }
}

#[test]
fn multiple_cursors_word_end_with_selection() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, bs: &dyn TextBuffer| {
        c.word_end_default(bs, true);
    };

    let progress = vec![
        "ala ma ko#ta\nkot ma# ale\npies sp#i\n#",
        "ala ma ko(ta]\nkot ma( ]ale\npies sp(i]\n#",
        "ala ma ko(ta\n]kot ma( ale]\npies sp(i\n]",
        "ala ma ko(ta\nkot] ma( ale\n]pies sp(i\n]",
        "ala ma ko(ta\nkot ]ma( ale\npies] sp(i\n]",
        "ala ma ko(ta\nkot ma]( ale\npies ]sp(i\n]",
        "ala ma ko(ta\nkot ma ](ale\npies spi](\n]",
        "ala ma ko(ta\nkot ma ale](\npies spi\n]",
        "ala ma ko(ta\nkot ma ale\n](pies spi\n]",
        "ala ma ko(ta\nkot ma ale\npies]( spi\n]",
        "ala ma ko(ta\nkot ma ale\npies ](spi\n]",
        "ala ma ko(ta\nkot ma ale\npies spi](\n]",
        "ala ma ko(ta\nkot ma ale\npies spi\n]",
    ];

    for i in 0..progress.len() - 1 {
        assert_eq!(apply_on_encoded_text_and_cursors(progress[i], f), progress[i + 1], "i: {}", i);
    }
}

#[test]
fn multiple_cursors_word_begin_with_selection() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, bs: &dyn TextBuffer| {
        c.word_begin_default(bs, true);
    };

    let progress = vec![
        "ala ma ko#ta\nkot ma# ale\npies sp#i\n#",
        "ala ma [ko)ta\nkot [ma) ale\npies [sp)i[\n)",
        "ala ma[ ko)ta\nkot[ ma) ale\npies[ )[spi\n)",
        "ala [ma ko)ta\n[kot ma) ale\n[pies)[ spi\n)",
        "ala[ ma ko)ta[\nkot ma) ale[\n)[pies spi\n)",
        "[ala ma )[kota\nkot ma) [ale)[\npies spi\n)",
        "[ala ma)[ kota\nkot ma)[ )[ale\npies spi\n)",
        "[ala )[ma kota\nkot )[ma)[ ale\npies spi\n)",
        "[ala)[ ma kota\nkot)[ )[ma ale\npies spi\n)",
        "[ala ma kota\n)[kot)[ ma ale\npies spi\n)",
        "[ala ma kota)[\n)[kot ma ale\npies spi\n)",
        "[ala ma )[kota)[\nkot ma ale\npies spi\n)",
        "[ala ma)[ )[kota\nkot ma ale\npies spi\n)",
        "[ala )[ma)[ kota\nkot ma ale\npies spi\n)",
        "[ala)[ )[ma kota\nkot ma ale\npies spi\n)",
        "[ala)[ ma kota\nkot ma ale\npies spi\n)",
        "[ala ma kota\nkot ma ale\npies spi\n)",
    ];

    for i in 0..progress.len() - 1 {
        assert_eq!(apply_on_encoded_text_and_cursors(progress[i], f), progress[i + 1], "i: {}", i);
    }
}

#[test]
fn single_cursor_to_flip_selection_bug_1() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, bs: &dyn TextBuffer| {
        c.move_vertically_by(bs, -1, true);
    };

    assert_eq!(apply_on_encoded_text_and_cursors("abcd\nabc(d\na]bcd", f), "abcd\na[bc)d\nabcd");
}

#[test]
fn single_cursor_to_flip_selection_bug_2() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, bs: &dyn TextBuffer| {
        c.move_vertically_by(bs, -1, true);
    };

    assert_eq!(apply_on_encoded_text_and_cursors("abcd\na(bc]d\nabcd", f), "abc[d\na)bcd\nabcd");
}
