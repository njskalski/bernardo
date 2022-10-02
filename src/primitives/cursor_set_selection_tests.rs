#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;

    use ropey::Rope;

    use crate::primitives::cursor_set::{Cursor, CursorSet, Selection};
    use crate::text::text_buffer::TextBuffer;

    // In this variant, a cursor is represented by a pair [ ) or ( ], with [ or ] marking the anchor.
// No overlaps allowed.
    pub fn text_to_buffer_cursors_with_selections(s: &str) -> (Rope, CursorSet) {
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

            if c == '#' {
                assert!(other_part.is_none(), "either # or pair : ( and ]");
                cursors.push(Cursor::new(idx));
                continue;
            }

            text.push(c);
            // println!("text: {} {:?}", text, cursors);
        }

        let cursors: Vec<Cursor> = cursors.iter().map(|a| (*a).into()).collect();
        (Rope::from(text), CursorSet::new(cursors))
    }

    pub fn apply_sel(input: &str, f: fn(&mut CursorSet, &dyn TextBuffer) -> ()) -> String {
        let (bs, mut cs) = text_to_buffer_cursors_with_selections(input);
        f(&mut cs, &bs);
        buffer_cursors_sel_to_text(&bs, &cs)
    }

    pub fn buffer_cursors_sel_to_text(b: &dyn TextBuffer, cs: &CursorSet) -> String {
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
                output.insert(idx, '#');
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

    #[test]
    fn apply_sel_works() {
        let f: fn(&mut CursorSet, &dyn TextBuffer) = |_c: &mut CursorSet, _b: &dyn TextBuffer| {};

        assert_eq!(apply_sel("text", f), "text");
        assert_eq!(apply_sel("te[xt)", f), "te[xt)");
        assert_eq!(apply_sel("[t)(ext]", f), "[t)(ext]");
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
            assert_eq!(apply_sel(expected_process[i], f), expected_process[i + 1]);
        }
    }

    #[test]
    fn home() {
        let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
            c.move_home(b, true);
        };

        // assert_eq!(apply_sel("text", f), "text");
        assert_eq!(apply_sel("text#", f), "[text)");
        assert_eq!(apply_sel("te#xt", f), "[te)xt");
        assert_eq!(apply_sel("a#aa\nbb#b\nccc#\n#", f), "[a)aa\n[bb)b\n[ccc)\n#");
        assert_eq!(apply_sel("a#aa\nb#b#b\nccc#\n#", f), "[a)aa\n[bb)b\n[ccc)\n#");
        assert_eq!(apply_sel("a#aa\nb#b#b\n#ccc#\n##", f), "[a)aa\n[bb)b\n[ccc)\n#");
    }

    #[test]
    fn end() {
        let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
            c.move_end(b, true);
        };

        // assert_eq!(apply_sel("text", f), "text");
        assert_eq!(apply_sel("text#", f), "text#");
        assert_eq!(apply_sel("te#xt", f), "te(xt]");
        assert_eq!(apply_sel("a#aa\nbb#b\nccc#\n#", f), "a(aa]\nbb(b]\nccc#\n#");
        assert_eq!(apply_sel("a#aa\nb#b#b\nccc#\n#", f), "a(aa]\nb(bb]\nccc#\n#");
        assert_eq!(apply_sel("a#aa\nb#b#b\n#ccc#\n##", f), "a(aa]\nb(bb]\n(ccc]\n#");
    }

    #[test]
    fn arrow_up_1() {
        let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
            c.move_vertically_by(b, -1, true);
        };

        // assert_eq!(apply_sel("text", f), "text");
        assert_eq!(apply_sel("text#", f), "[text)");
        assert_eq!(apply_sel("te#xt", f), "[te)xt");

        assert_eq!(apply_sel("lin#e1\nline2\nli#ne3", f), "[lin)e1\nli[ne2\nli)ne3");
        assert_eq!(apply_sel("lin#e1\nline2\nli#ne3\n#", f), "[lin)e1\nli[ne2\n)[line3\n)");
    }

    #[test]
    fn arrow_up_2() {
        let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
            c.move_vertically_by(b, -1, true);
        };

        assert_eq!(apply_sel("line1\nline2\nli#ne3", f), "line1\nli[ne2\nli)ne3");
        assert_eq!(apply_sel("line1\nli[ne2\nli)ne3", f), "li[ne1\nline2\nli)ne3");
        assert_eq!(apply_sel("li[ne1\nline2\nli)ne3", f), "[line1\nline2\nli)ne3");
    }

    #[test]
    fn arrow_down_1() {
        let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
            c.move_vertically_by(b, 1, true);
        };

        // assert_eq!(apply_sel("text", f), "text");
        assert_eq!(apply_sel("#text", f), "(text]");
        assert_eq!(apply_sel("te#xt", f), "te(xt]");

        assert_eq!(apply_sel("lin#e1\nline2\nli#ne3", f), "lin(e1\nlin]e2\nli(ne3]");
        assert_eq!(apply_sel("lin#e1\nline2\nli#ne3\n#", f), "lin(e1\nlin]e2\nli(ne3\n]");
    }

    #[test]
    fn arrow_down_2() {
        let f: fn(&mut CursorSet, &dyn TextBuffer) = |c: &mut CursorSet, b: &dyn TextBuffer| {
            c.move_vertically_by(b, 1, true);
        };

        assert_eq!(apply_sel("li#ne1\nline2\nline3", f), "li(ne1\nli]ne2\nline3");
        assert_eq!(apply_sel("li(ne1\nli]ne2\nline3", f), "li(ne1\nline2\nli]ne3");
        assert_eq!(apply_sel("li(ne1\nline2\nli]ne3", f), "li(ne1\nline2\nline3]");
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
            assert_eq!(apply_sel(progress[i], f), progress[i + 1], "i: {}", i);
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
            assert_eq!(apply_sel(progress[i], f), progress[i + 1], "i: {}", i);
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
            assert_eq!(apply_sel(progress[i], f), progress[i + 1], "i: {}", i);
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
            assert_eq!(apply_sel(progress[i], f), progress[i + 1], "i: {}", i);
        }
    }
}