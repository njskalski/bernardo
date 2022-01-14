use log::error;
use ropey::Rope;

use crate::primitives::cursor_set::{Cursor, CursorSet, Selection};
use crate::text::buffer::{Buffer, buffer_to_string};

// In this variant, a cursor is represented by a pair [ ) or ( ], with [ or ] marking the anchor.
// No overlaps allowed.
fn text_to_buffer_cursors_with_selections(s: &str) -> (Rope, CursorSet) {
    let mut text = String::new();
    let mut cursors: Vec<Cursor> = vec![];
    let mut other_part: Option<usize> = None;

    for c in s.chars() {
        let idx = text.len();
        if c == '[' || c == ']' {
            match other_part {
                None => other_part = Some(idx),
                Some(other_idx) => {
                    cursors.push(
                        Cursor::new(idx).with_selection(Selection::new(other_idx, idx))
                    );
                    other_part = None;
                }
            }
            continue;
        }

        if c == '(' || c == ')' {
            match other_part {
                None => other_part = Some(idx),
                Some(other_idx) => {
                    cursors.push(
                        Cursor::new(other_idx).with_selection(Selection::new(other_idx, idx))
                    );
                    other_part = None;
                }
            };
            continue;
        }

        text.push(c);
        // println!("text: {} {:?}", text, cursors);
    }

    let cursors: Vec<Cursor> = cursors.iter().map(|a| (*a).into()).collect();
    (Rope::from(text), CursorSet::new(cursors))
}

fn apply_sel(input: &str, f: fn(&mut CursorSet, &dyn Buffer) -> ()) -> String {
    let (bs, mut cs) = text_to_buffer_cursors_with_selections(input);
    f(&mut cs, &bs);
    buffer_cursors_sel_to_text(&bs, &cs)
}

fn buffer_cursors_sel_to_text(b: &dyn Buffer, cs: &CursorSet) -> String {
    // first we validate there is no overlaps. I initially wanted to sort beginnings and ends, but
    // since ends are exclusive, the false-positives could appear this way. So I'll just color
    // the vector.

    let mut colors: Vec<Option<usize>> = Vec::new();
    colors.resize(b.len_chars() + 1, None);

    for (cursor_idx, cursor) in cs.iter().enumerate() {
        match cursor.s {
            Some(sel) => {
                for idx in sel.b..sel.e {
                    assert_eq!(colors[idx], None, "cursor {} collides with {:?}", cursor_idx, colors[idx].unwrap());
                    colors[idx] = Some(cursor_idx);
                }
            }
            None => {}
        };
    }

    // ok, so now we know we have no overlaps. Since I already have the colors, I will use them
    // to reconstruct the coding string. I just have to remember to do it from the end, as any
    // insertion will invalidate all subsequent indices.

    let mut output = buffer_to_string(b);
    // cursor_idx (aka color), pos
    let mut current_cursor_idx: Option<usize> = None;

    for idx in (0..colors.len()).rev() {
        println!("{}: ci {:?} cc {:?} output {}", idx, colors[idx], current_cursor_idx, output);

        match (colors[idx], current_cursor_idx) {
            (Some(cursor_idx), None) => {
                // a cursor is ending here
                let end_pos = idx + 1; // the "end" is not inclusive, so we add +1.
                current_cursor_idx = Some(cursor_idx);
                let cursor = cs.set()[cursor_idx];

                if cursor.a == end_pos {
                    output.insert(end_pos, ']');
                } else {
                    output.insert(end_pos, ')');
                }
            },
            (Some(new_cursor_idx), Some(prev_cursor_idx)) if new_cursor_idx != prev_cursor_idx => {
                // this is hard, everything is selected, but the cursor changes.
                // first, let's close the previous one. This will be a "beginning", since we're walking backwards.

                let old_cursor = cs.set()[prev_cursor_idx];
                // beginnings are inclusive, but now we're over PREVIOUS character, the one that preceeds the end
                // of "previous" cursor (that has higher indices).

                let begin_pos = idx;
                if old_cursor.a == begin_pos {
                    output.insert(begin_pos, '[');
                } else {
                    output.insert(begin_pos, '(');
                }

                // now, let's open a new cursor
                // cursor_idx (aka color), pos
                current_cursor_idx = Some(new_cursor_idx);
            }
            (None, Some(prev_cursor_idx)) => {
                // here we just copy-paste from above
                let old_cursor = cs.set()[prev_cursor_idx];
                let begin_pos = idx;
                if old_cursor.a == begin_pos {
                    output.insert(begin_pos, '[');
                } else {
                    output.insert(begin_pos, '(');
                }
                // but clear the "current_cursor"
                current_cursor_idx = None;
            }
            _ => {} // None, None is noop.
        }
    }

    println!("after : cc {:?} output {}", current_cursor_idx, output);

    // handling cursor starting in 0
    match current_cursor_idx {
        Some(cursor_idx) => {
            let cursor = cs.set()[cursor_idx];
            if cursor.a == 0 {
                output.insert(0, '[');
            } else {
                output.insert(0, '(');
            }
        }
        None => {}
    }

    output
}

#[test]
fn test_text_to_buffer_cursors_with_selections_1() {
    let (text, cursors) = text_to_buffer_cursors_with_selections("te[xt)");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 1);
    assert_eq!(cursors.set()[0].a, 2);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(2, 4)));
}

#[test]
fn test_text_to_buffer_cursors_with_selections_2() {
    let (text, cursors) = text_to_buffer_cursors_with_selections("te(xt]");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 1);
    assert_eq!(cursors.set()[0].a, 4);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(2, 4)));
}

#[test]
fn test_text_to_buffer_cursors_with_selections_3() {
    let (text, cursors) = text_to_buffer_cursors_with_selections("(t]e(xt]");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 2);
    assert_eq!(cursors.set()[0].a, 1);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(0, 1)));
    assert_eq!(cursors.set()[1].a, 4);
    assert_eq!(cursors.set()[1].s, Some(Selection::new(2, 4)));
}

#[test]
fn test_text_to_buffer_cursors_with_selections_4() {
    let (text, cursors) = text_to_buffer_cursors_with_selections("(te](xt]");
    assert_eq!(text, "text");
    assert_eq!(cursors.set().len(), 2);
    assert_eq!(cursors.set()[0].a, 2);
    assert_eq!(cursors.set()[0].s, Some(Selection::new(0, 2)));
    assert_eq!(cursors.set()[1].a, 4);
    assert_eq!(cursors.set()[1].s, Some(Selection::new(2, 4)));
}

#[test]
fn test_buffer_cursors_sel_to_text_0() {
    let text = buffer_cursors_sel_to_text(&Rope::from("text"), &CursorSet::new(
        vec![]
    ));

    assert_eq!(text, "text");
}

#[test]
fn test_buffer_cursors_sel_to_text_1() {
    let text = buffer_cursors_sel_to_text(&Rope::from("text"), &CursorSet::new(
        vec![
            Cursor::new(0).with_selection(Selection::new(0, 2)),
        ]
    ));

    assert_eq!(text, "[te)xt");
}

#[test]
fn test_buffer_cursors_sel_to_text_2() {
    let text = buffer_cursors_sel_to_text(&Rope::from("text"), &CursorSet::new(
        vec![
            Cursor::new(0).with_selection(Selection::new(0, 2)),
            Cursor::new(2).with_selection(Selection::new(2, 4)),
        ]
    ));

    assert_eq!(text, "[te)[xt)");
}

// #[test]
// fn apply_sel_works() {
//     let f: fn(&mut CursorSet, &dyn Buffer) = |_c: &mut CursorSet, _b: &dyn Buffer| {};
//
//     assert_eq!(apply_sel("text", f), "text");
//     assert_eq!(apply_sel("te[xt)", f), "te[xt)");
//     assert_eq!(apply_sel("[t)(ext]", f), "[t)(ext]");
// }