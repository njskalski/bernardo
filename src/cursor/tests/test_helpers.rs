use std::collections::HashSet;

use ropey::Rope;

use crate::cursor::cursor::{Cursor, Selection};
use crate::cursor::cursor_set::CursorSet;
use crate::text::text_buffer::TextBuffer;

/*
This method takes a string encoding a text with some cursors, and returns solid types representing
them.

Legend:
- '.' is an empty character that will be skipped.
- # is a cursor with no selection
- [ ) is a cursor with selection, anchored on the left
- ( ] is a cursor with selection, anchored on the right.

Of course lower index is inclusive, higher index is exclusive.

Examples:

1) "ala#" is text "ala" and single cursor behind letter a, at index 3, no selection.
2) "a[la)" is text "ala", single cursor that encompasses "la", with anchor at element 1 and
    selection of length 2.
3) "a.la ma# ko#ta" is "ala ma kota" with two cursors, one at 6 and one at 9, no selections.
 */
pub fn decode_text_and_cursors(s: &str) -> (Rope, CursorSet) {
    let mut text = String::new();
    let mut cursors: Vec<Cursor> = vec![];
    let mut other_part: Option<usize> = None;

    for c in s.chars() {
        if c == '.' {
            continue;
        }

        let idx = text.len();
        if c == '[' || c == ']' {
            match other_part {
                None => other_part = Some(idx),
                Some(other_idx) => {
                    cursors.push(Cursor::new(idx).with_selection(Selection::new(other_idx, idx)));
                    other_part = None;
                }
            }
            continue;
        }

        if c == '(' || c == ')' {
            match other_part {
                None => other_part = Some(idx),
                Some(other_idx) => {
                    cursors.push(Cursor::new(other_idx).with_selection(Selection::new(other_idx, idx)));
                    other_part = None;
                }
            };
            continue;
        }

        if c == '#' {
            assert!(other_part.is_none(), "either # or pair : ( and ]");
            cursors.push(Cursor::new(idx));
            continue;
        }

        text.push(c);
        // println!("text: {} {:?}", text, cursors);
    }

    let cursors: Vec<Cursor> = cursors.iter().map(|a| (*a).into()).collect();
    let rope = Rope::from(text);
    let cs = CursorSet::new(cursors);

    //sanity check
    assert_cursors_are_within_text(&rope, &cs);

    (rope, cs)
}

pub fn assert_cursors_are_within_text(b: &dyn TextBuffer, cs: &CursorSet) {
    let b_len_chars = b.len_chars();
    for c in cs.iter() {
        debug_assert!(c.a <= b_len_chars);
        match c.s {
            None => {}
            Some(s) => {
                debug_assert!(s.e <= b_len_chars);
            }
        }
    }
}

pub fn common_apply(input: &str, f: fn(&mut CursorSet, &dyn TextBuffer) -> ()) -> String {
    let (bs, mut cs) = decode_text_and_cursors(input);
    f(&mut cs, &bs);
    encode_cursors_and_text(&bs, &cs)
}

pub fn encode_cursors_and_text(b: &dyn TextBuffer, cs: &CursorSet) -> String {
    // first let's check there's no impossible orders
    assert_cursors_are_within_text(b, cs);

    // first we validate there is no overlaps. I initially wanted to sort beginnings and ends, but
    // since ends are exclusive, the false-positives could appear this way. So I'll just color
    // the vector.

    let debug_buffer_as_string = b.to_string();
    assert_eq!(debug_buffer_as_string.len(), b.len_chars());

    let mut colors: Vec<Option<usize>> = Vec::new();

    // the +2 is because the last cursor can point at NON EXISTENT character
    colors.resize(b.len_chars() + 2, None);

    for (cursor_idx, cursor) in cs.iter().enumerate() {
        match cursor.s {
            Some(sel) => {
                for idx in sel.b..sel.e {
                    assert_eq!(colors[idx], None, "cursor {} collides with {:?}", cursor_idx, colors[idx].unwrap());
                    colors[idx] = Some(cursor_idx);
                }
            }
            None => {
                assert_eq!(
                    colors[cursor.a],
                    None,
                    "cursor {} collides with {:?} by anchor",
                    cursor_idx,
                    colors[cursor.a].unwrap()
                );
            }
        };
    }

    // ok, so now we know we have no overlaps. Since I already have the colors, I will use them
    // to reconstruct the coding string. I just have to remember to do it from the end, as any
    // insertion will invalidate all subsequent indices.

    let mut output = b.to_string();
    // cursor_idx (aka color), pos
    let mut current_cursor_idx: Option<usize> = None;

    fn add_cursor_end(end_pos: usize, cursor: &Cursor, output: &mut String) {
        assert_eq!(cursor.s.map(|s| s.e), Some(end_pos));
        if cursor.a == end_pos {
            output.insert(end_pos, ']');
        } else {
            output.insert(end_pos, ')');
        }
    }

    fn add_cursor_begin(begin_pos: usize, cursor: &Cursor, output: &mut String) {
        assert_eq!(cursor.s.map(|s| s.b), Some(begin_pos));
        if cursor.a == begin_pos {
            output.insert(begin_pos, '[');
        } else {
            output.insert(begin_pos, '(');
        }
    }

    let lone_cursors: HashSet<usize> = cs.set().iter().filter(|c| c.s.is_none()).map(|c| c.a).collect();

    for idx in (0..colors.len()).rev() {
        // println!("{}: ci {:?} cc {:?} output {}", idx, colors[idx], current_cursor_idx, output);

        match (colors[idx], current_cursor_idx) {
            (Some(cursor_idx), None) => {
                // a cursor is ending here
                let end_pos = idx + 1; // the "end" is not inclusive, so we add +1.
                add_cursor_end(end_pos, &cs.set()[cursor_idx], &mut output);
                current_cursor_idx = Some(cursor_idx);
            }
            (Some(new_cursor_idx), Some(prev_cursor_idx)) if new_cursor_idx != prev_cursor_idx => {
                // this is hard, everything is selected, but the cursor changes.
                // first, let's close the previous one. This will be a "beginning", since we're walking backwards.
                let begin_end_pos = idx + 1;
                add_cursor_begin(begin_end_pos, &cs.set()[prev_cursor_idx], &mut output);
                current_cursor_idx = Some(new_cursor_idx);
                add_cursor_end(begin_end_pos, &cs.set()[new_cursor_idx], &mut output);
            }
            (None, Some(prev_cursor_idx)) => {
                let begin_pos = idx + 1;
                add_cursor_begin(begin_pos, &cs.set()[prev_cursor_idx], &mut output);
                current_cursor_idx = None;
            }
            _ => {} // None, None is noop.
        }

        if lone_cursors.contains(&idx) {
            if output.len() > idx {
                output.insert(idx, '#');
            } else {
                assert!(output.len() == idx);
                output.push('#');
            }
        }
    }

    // println!("after : cc {:?} output {}", current_cursor_idx, output);

    // handling cursor starting in 0
    match current_cursor_idx {
        Some(cursor_idx) => {
            add_cursor_begin(0, &cs.set()[cursor_idx], &mut output);
            // current_cursor_idx = None;
        }
        None => {}
    }

    output
}

// these are the tests of testing framework. It's complicated.

#[test]
fn decode_text_and_cursors_1() {
    let (text, cursors) = decode_text_and_cursors("te[xt)");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 1);
    assert_eq!(cursors.set()[0].a, 2);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(2, 4)));
}

#[test]
fn decode_text_and_cursors_2() {
    let (text, cursors) = decode_text_and_cursors("te(xt]");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 1);
    assert_eq!(cursors.set()[0].a, 4);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(2, 4)));
}

#[test]
fn decode_text_and_cursors_3() {
    let (text, cursors) = decode_text_and_cursors("(t]e(xt]");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 2);
    assert_eq!(cursors.set()[0].a, 1);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(0, 1)));
    assert_eq!(cursors.set()[1].a, 4);
    assert_eq!(cursors.set()[1].s, Some(Selection::new(2, 4)));
}

#[test]
fn decode_text_and_cursors_4() {
    let (text, cursors) = decode_text_and_cursors("(te](xt]");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 2);
    assert_eq!(cursors.set()[0].a, 2);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(0, 2)));
    assert_eq!(cursors.set()[1].a, 4);
    assert_eq!(cursors.set()[1].s, Some(Selection::new(2, 4)));
}

#[test]
fn decode_text_and_cursors_5() {
    let (text, cursors) = decode_text_and_cursors("text#");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 1);
    assert_eq!(cursors.set()[0].a, 4);
    assert_eq!(cursors.set()[0].s, None);
}

#[test]
fn encode_text_and_cursors_0() {
    let text = encode_cursors_and_text(&Rope::from("text"), &CursorSet::new(vec![]));

    assert_eq!(text, "text");
}

#[test]
fn encode_text_and_cursors_1() {
    let text = encode_cursors_and_text(
        &Rope::from("text"),
        &CursorSet::new(vec![Cursor::new(0).with_selection(Selection::new(0, 2))]),
    );

    assert_eq!(text, "[te)xt");
}

#[test]
fn encode_text_and_cursors_2() {
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
fn encode_text_and_cursors_3() {
    let text = encode_cursors_and_text(&Rope::from("text\n"), &CursorSet::new(vec![Cursor::new(5)]));

    assert_eq!(text, "text\n#");
}

#[test]
fn verify_encode_decode_identity() {
    let f: fn(&mut CursorSet, &dyn TextBuffer) = |_c: &mut CursorSet, _b: &dyn TextBuffer| {};

    assert_eq!(common_apply("text", f), "text");
    assert_eq!(common_apply("te[xt)", f), "te[xt)");
    assert_eq!(common_apply("[t)(ext]", f), "[t)(ext]");
}
