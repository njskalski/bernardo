// Copyright 2018 Google LLC, 2021 Andrzej J Skalski
// original file was part of sly-editor, at
// https://github.com/njskalski/sly-editor/blob/master/src/cursor_set.rs
// it's copyrighted by Google LLC at Apache 2 license.
// This version is updated by Andrzej J Skalski, and licensed GPLv3.

// encoding # as cursor anchor
// it points to a character that will be replaced/preceded, not succeeded

#[cfg(test)]
pub mod tests {
    use std::borrow::Borrow;

    use ropey::Rope;

    // use serde::de::Unexpected::Str;
    use crate::primitives::cursor_set::{Cursor, CursorSet, Selection};

    pub fn text_to_buffer_cursors(s: &str) -> (Rope, CursorSet) {
        let mut cursors: Vec<usize> = vec![];
        let mut text = String::new();

        dbg!(s);

        for c in s.chars() {
            if c == '#' {
                cursors.push(text.len());
                continue;
            }

            text.push(c);
        }

        dbg!(&cursors);

        let cursors: Vec<Cursor> = cursors.iter().map(|a| (*a).into()).collect();

        (Rope::from(text), CursorSet::new(cursors))
    }

    // cursors start with [ or # and end with ] or #, having exactly one #, and exactly one of [ or ]
    pub fn text_to_buffer_cursors_with_selections(s: &str) -> (Rope, CursorSet) {
        let text = String::new();

        let mut cursors: Vec<Cursor> = vec![];

        dbg!(s);

        let mut other_part: Option<usize> = None;

        for c in s.chars() {
            let pos = text.len();
            if c == '#' {
                match other_part {
                    None => other_part = Some(pos),
                    Some(idx) => {
                        cursors.push(
                            Cursor::new(pos).with_selection(Selection::new(idx, pos))
                        );
                    }
                }
                continue;
            }

            if c == '[' || c == ']' {
                match other_part {
                    None => other_part = Some(pos),
                    Some(idx) => {
                        cursors.push(
                            Cursor::new(idx).with_selection(Selection::new(idx, pos))
                        );
                    }
                };
                continue;
            }
        }

        dbg!(&cursors);

        let cursors: Vec<Cursor> = cursors.iter().map(|a| (*a).into()).collect();

        (Rope::from(text), CursorSet::new(cursors))
    }

    pub fn buffer_cursors_to_text<T: Borrow<Rope>>(b: T, cs: &CursorSet) -> String {
        let mut output = String::new();
        let mut anchors: Vec<usize> = cs.set().iter().map(|c| c.a).collect();
        anchors.sort();

        let buffer = b.borrow().to_string();

        let mut prev: usize = 0;
        for a in anchors {
            if prev < buffer.len() {
                output.push_str(&buffer[prev..std::cmp::min(a, buffer.len())]);
            }

            if output.chars().last() == Some('#') {
                continue; // reducing
            }
            output.push_str("#");
            prev = a;
        }
        if prev < buffer.len() {
            output.push_str(&buffer[prev..]);
        }

        output
    }

    fn a_to_c(anchors: Vec<usize>) -> CursorSet {
        CursorSet::new(anchors.iter().map(|a| (*a).into()).collect())
    }

    #[test]
    fn get_buffer_test_1() {
        let (bs, cs) = text_to_buffer_cursors("te#xt");
        let text = bs.to_string();

        assert_eq!(text, "text".to_owned());
        assert_eq!(
            cs.set().iter().map(|c| c.a).collect::<Vec<usize>>(),
            vec![2]
        );
    }

    #[test]
    fn get_buffer_test_2() {
        let (bs, cs) = text_to_buffer_cursors("#text#");
        let text = bs.to_string();

        assert_eq!(text, "text".to_owned());
        assert_eq!(
            cs.set().iter().map(|c| c.a).collect::<Vec<usize>>(),
            vec![0, 4]
        );
    }

    #[test]
    fn buffer_cursors_to_text_1() {
        let cursors = a_to_c(vec![2]);
        let buffer = Rope::from_str("text");

        let output = buffer_cursors_to_text(&buffer, &cursors);

        assert_eq!(output, "te#xt".to_owned());
    }

    #[test]
    fn buffer_cursors_to_text_2() {
        let cursors = a_to_c(vec![0, 4]);
        let buffer = Rope::from_str("text");

        let output = buffer_cursors_to_text(&buffer, &cursors);

        assert_eq!(output, "#text#".to_owned());
    }

    #[test]
    fn buffer_cursors_to_text_3() {
        let cursors = a_to_c(vec![1]);
        let buffer = Rope::from_str("text");

        let output = buffer_cursors_to_text(&buffer, &cursors);

        assert_eq!(output, "t#ext".to_owned());
    }

    #[test]
    fn text_to_buffer_cursors_and_back() {
        let text = concat!(
        "t#here was a man\n",
        "called paul\n",
        "#who went to a fancy\n",
        "dr#ess ball\n",
        "he just went for fun\n",
        "dressed up as bone\n",
        "and dog ea#t h#im up in the# hall.\n"
        );

        let (buffer, cursors) = text_to_buffer_cursors(text);
        let back = buffer_cursors_to_text(&buffer, &cursors);

        assert_eq!(text, back);
    }

    fn apply(input: &str, f: fn(&mut CursorSet, &Rope) -> ()) -> String {
        let (bs, mut cs) = text_to_buffer_cursors(input);
        f(&mut cs, &bs);
        buffer_cursors_to_text(&bs, &cs)
    }

    #[test]
    fn one_cursor_move_left() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, _| {
            c.move_left(false);
        };

        // assert_eq!(apply("text", f), "text");
        assert_eq!(apply("te#xt", f), "t#ext");
        assert_eq!(apply("t#ext", f), "#text");
        assert_eq!(apply("#text", f), "#text");
        assert_eq!(apply("text\n#", f), "text#\n");
    }

    #[test]
    fn one_cursor_move_left_some() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, _| {
            c.move_left_by(3, false);
        };

        // assert_eq!(apply("text", f), "text");
        assert_eq!(apply("te#xt", f), "#text");
        assert_eq!(apply("t#ext", f), "#text");
        assert_eq!(apply("#text", f), "#text");
        assert_eq!(apply("text\n#", f), "te#xt\n");
    }

    #[test]
    fn multiple_cursor_move_left() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, _| {
            c.move_left(false);
        };

        assert_eq!(apply("te#x#t", f), "t#e#xt");
        assert_eq!(apply("#t#ext", f), "#text");
        assert_eq!(apply("#text\n#", f), "#text#\n");
    }

    #[test]
    fn multiple_cursor_move_left_some() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, _| {
            c.move_left_by(3, false);
        };

        assert_eq!(apply("te#x#t", f), "#text");
        assert_eq!(apply("#t#ext", f), "#text");
        assert_eq!(apply("#text\n#", f), "#te#xt\n");
    }

    #[test]
    fn one_cursor_move_right() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.move_right(bs, false);
        };

        // assert_eq!(apply("text", f), "text");
        assert_eq!(apply("te#xt", f), "tex#t");
        assert_eq!(apply("t#ext", f), "te#xt");
        assert_eq!(apply("#text", f), "t#ext");
        assert_eq!(apply("text\n#", f), "text\n#");
    }

    #[test]
    fn one_cursor_move_right_some() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.move_right_by(bs, 3, false);
        };

        // assert_eq!(apply("text", f), "text");
        assert_eq!(apply("te#xt", f), "text#");
        assert_eq!(apply("t#ext", f), "text#");
        assert_eq!(apply("#text", f), "tex#t");
        assert_eq!(apply("text\n#", f), "text\n#");
    }

    #[test]
    fn multiple_cursor_move_right() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.move_right(bs, false);
        };

        assert_eq!(apply("te#x#t", f), "tex#t#");
        assert_eq!(apply("#t#ext", f), "t#e#xt");
        assert_eq!(apply("#text\n#", f), "t#ext\n#");
        assert_eq!(apply("text#\n#", f), "text\n#");
    }

    #[test]
    fn multiple_cursor_move_right_some() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.move_right_by(bs, 3, false);
        };

        assert_eq!(apply("te#x#t", f), "text#");
        assert_eq!(apply("#t#ext", f), "tex#t#");
        assert_eq!(apply("#text\n#", f), "tex#t\n#");
        assert_eq!(apply("text#\n#", f), "text\n#");
    }

    #[test]
    fn single_cursor_move_down_by_1() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.move_vertically_by(bs, 1, false);
        };

        // noop
        // assert_eq!(apply("aaaa\nbbbb", f), "aaaa\nbbbb");

        // moving down the line
        assert_eq!(apply("a#aaa\nbbbb", f), "aaaa\nb#bbb");
        assert_eq!(apply("aaaa#\nbbbb\ncccc", f), "aaaa\nbbbb#\ncccc");
        assert_eq!(apply("aaaa#\nbbbb", f), "aaaa\nbbbb#");
        assert_eq!(apply("aaaa\nbb#bb", f), "aaaa\nbbbb#");

        // moving withing the line
        assert_eq!(apply("te#x#t", f), "text#");
        assert_eq!(apply("#t#ext", f), "text#");
        assert_eq!(apply("#text\n#", f), "text\n#");
        assert_eq!(apply("text#\n#", f), "text\n#");

        // moving between lines varying in length
        assert_eq!(apply("3#33\n22\n1", f), "333\n2#2\n1");
        assert_eq!(apply("33#3\n22\n1", f), "333\n22#\n1");
        assert_eq!(apply("333#\n22\n1", f), "333\n22#\n1");
        assert_eq!(apply("333\n#22\n1", f), "333\n22\n#1");
        assert_eq!(apply("333\n2#2\n1", f), "333\n22\n1#");
    }

    #[test]
    fn single_cursor_move_down_by_2() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.move_vertically_by(bs, 2, false);
        };

        // moving down the line
        assert_eq!(apply("aa#aa\nbbbb\ncccc", f), "aaaa\nbbbb\ncc#cc");
        assert_eq!(
            apply("aaaa\nbbb#b\ncccc\ndddd", f),
            "aaaa\nbbbb\ncccc\nddd#d"
        );
        assert_eq!(
            apply("aaaa\nbbbb\nc#ccc\ndddd", f),
            "aaaa\nbbbb\ncccc\ndddd#"
        );
        assert_eq!(
            apply("aaaa\nbbbb\nc#ccc\ndddd\n", f),
            "aaaa\nbbbb\ncccc\ndddd\n#"
        );
        assert_eq!(
            apply("aa#a#a\nbbbb\ncccc\ndddd\n", f),
            "aaaa\nbbbb\ncc#c#c\ndddd\n"
        );
        assert_eq!(
            apply("aaaa\nb#b#b#b\ncccc\ndddd\n", f),
            "aaaa\nbbbb\ncccc\nd#d#d#d\n"
        );
        assert_eq!(
            apply("a#a#a#a\nbbbb\ncccc\ndddd\n", f),
            "aaaa\nbbbb\nc#c#c#c\ndddd\n"
        );

        //    // moving withing the line
        assert_eq!(apply("a#aaa\nbbbb", f), "aaaa\nbbbb#");
        assert_eq!(apply("aaaa#\nbbbb", f), "aaaa\nbbbb#");
        assert_eq!(apply("aaaa\nbb#bb", f), "aaaa\nbbbb#");
    }

    #[test]
    fn single_cursor_move_down_by_some() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.move_vertically_by(bs, 3, false);
        };

        {
            let text = concat!(
            "t#here was a man\n",
            "called paul\n",
            "who went to a fancy\n",
            "dress ball\n",
            "he just went for fun\n",
            "dressed up as bone\n",
            "and dog eat him up in the hall.\n"
            );

            let new_text = concat!(
            "there was a man\n",
            "called paul\n",
            "who went to a fancy\n",
            "d#ress ball\n",
            "he just went for fun\n",
            "dressed up as bone\n",
            "and dog eat him up in the hall.\n"
            );

            assert_eq!(apply(text, f), new_text);
        }

        {
            let text = concat!(
            "t#here was a ma#n\n",
            "calle#d paul\n",
            "who went to a fancy\n",
            "dress ball\n",
            "he just went for fun\n",
            "dressed up as bone\n",
            "and dog eat him up in the hall.\n"
            );

            let new_text = concat!(
            "there was a man\n",
            "called paul\n",
            "who went to a fancy\n",
            "d#ress ball#\n",
            "he ju#st went for fun\n",
            "dressed up as bone\n",
            "and dog eat him up in the hall.\n"
            );

            assert_eq!(apply(text, f), new_text);
        }
    }

    #[test]
    fn single_cursor_move_up_by_1() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.move_vertically_by(bs, -1, false);
        };

        // noop
        // assert_eq!(apply("aaaa\nbbbb", f), "aaaa\nbbbb");

        assert_eq!(apply("a#aaa\nbbbb", f), "#aaaa\nbbbb");
        assert_eq!(apply("aaaa#\nbbbb", f), "#aaaa\nbbbb");
        assert_eq!(apply("aaaa\nbbbb\ncccc#", f), "aaaa\nbbbb#\ncccc");

        assert_eq!(apply("aaaa\nbb#bb", f), "aa#aa\nbbbb");

        assert_eq!(apply("te#x#t", f), "#text");
        assert_eq!(apply("#t#ext", f), "#text");
        assert_eq!(apply("#text\n#", f), "#text\n");
        assert_eq!(apply("text#\n#", f), "#text\n");

        assert_eq!(apply("3#33\n22\n1", f), "#333\n22\n1");
        assert_eq!(apply("333\n#22\n1", f), "#333\n22\n1");
        assert_eq!(apply("333\n22#\n1", f), "33#3\n22\n1");

        assert_eq!(apply("1\n22\n3#33", f), "1\n2#2\n333");
        assert_eq!(apply("1\n22\n33#3", f), "1\n22#\n333");
        assert_eq!(apply("1\n22\n333#", f), "1\n22#\n333");
    }

    #[test]
    fn single_cursor_move_up_by_2() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.move_vertically_by(bs, -2, false);
        };

        assert_eq!(apply("aaaa\nbbbb\ncc#cc", f), "aa#aa\nbbbb\ncccc");
        assert_eq!(
            apply("aaaa\nbbbb\ncccc\nddd#d", f),
            "aaaa\nbbb#b\ncccc\ndddd"
        );
        assert_eq!(
            apply("aaaa\nbb#bb\ncccc\ndddd", f),
            "#aaaa\nbbbb\ncccc\ndddd"
        );
        assert_eq!(
            apply("aaaa\nbbbb\ncccc\ndddd\n#", f),
            "aaaa\nbbbb\n#cccc\ndddd\n"
        );
        assert_eq!(
            apply("aaaa\nbbbb\ncc#c#c\ndddd\n", f),
            "aa#a#a\nbbbb\ncccc\ndddd\n"
        );
        assert_eq!(
            apply("aaaa\nbbbb\ncccc\nd#d#d#d\n", f),
            "aaaa\nb#b#b#b\ncccc\ndddd\n"
        );
        assert_eq!(
            apply("aaaa\nbbbb\nc#c#c#c\ndddd\n", f),
            "a#a#a#a\nbbbb\ncccc\ndddd\n"
        );

        assert_eq!(apply("aaaa\nbb#bb", f), "#aaaa\nbbbb");
        assert_eq!(apply("aaaa\nbbbb#", f), "#aaaa\nbbbb");
        assert_eq!(apply("aaaa\nbbb#b", f), "#aaaa\nbbbb");
    }

    #[test]
    fn single_cursor_move_up_by_some() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.move_vertically_by(bs, -3, false);
        };

        {
            let text = concat!(
            "t#here was a man\n",
            "called paul\n",
            "who went to a fancy\n",
            "dress ball\n",
            "he just went for fun\n",
            "d#ressed up as bone\n",
            "and dog eat him up in the hall.#\n"
            );

            let new_text = concat!(
            "#there was a man\n",
            "called paul\n",
            "w#ho went to a fancy\n",
            "dress ball#\n",
            "he just went for fun\n",
            "dressed up as bone\n",
            "and dog eat him up in the hall.\n"
            );

            assert_eq!(apply(text, f), new_text);
        }
    }


    #[test]
    fn single_cursor_to_move_up_bug_1() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.move_vertically_by(bs, -1, false);
            c.move_vertically_by(bs, -1, false);
            c.move_vertically_by(bs, -1, false);
        };

        let x = "asdf\nasd\n\ndsafsdf\nfdsafds#";
        let y = "asdf\nasd#\n\ndsafsdf\nfdsafds";

        assert_eq!(apply(x, f), y);
    }

    #[test]
    fn single_cursor_word_begin() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.word_begin_default(bs, false);
        };

        let progress = vec![
            "ala ma ko#ta",
            "ala ma #kota",
            "ala ma# kota",
            "ala #ma kota",
            "ala# ma kota",
            "#ala ma kota",
            "#ala ma kota",
            "#ala ma kota",
        ];

        for i in 0..progress.len() - 1 {
            assert_eq!(apply(progress[i], f), progress[i + 1], "i: {}", i);
        }
    }

    #[test]
    fn multiple_cursors_word_begin() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.word_begin_default(bs, false);
        };

        let progress = vec![
            "ala ma ko#ta\nkot ma# ale\npies sp#i\n#",
            "ala ma #kota\nkot #ma ale\npies #spi#\n",
            "ala ma# kota\nkot# ma ale\npies# #spi\n",
            "ala #ma kota\n#kot ma ale\n#pies# spi\n",
            "ala# ma kota#\nkot ma ale#\n#pies spi\n",
            "#ala ma #kota\nkot ma #ale#\npies spi\n",
            "#ala ma# kota\nkot ma# #ale\npies spi\n",
            "#ala #ma kota\nkot #ma# ale\npies spi\n",
            "#ala# ma kota\nkot# #ma ale\npies spi\n",
            "#ala ma kota\n#kot# ma ale\npies spi\n",
            "#ala ma kota#\n#kot ma ale\npies spi\n",
            "#ala ma #kota#\nkot ma ale\npies spi\n",
            "#ala ma# #kota\nkot ma ale\npies spi\n",
            "#ala #ma# kota\nkot ma ale\npies spi\n",
            "#ala# #ma kota\nkot ma ale\npies spi\n",
            "#ala# ma kota\nkot ma ale\npies spi\n",
            "#ala ma kota\nkot ma ale\npies spi\n",
        ];

        for i in 0..progress.len() - 1 {
            assert_eq!(apply(progress[i], f), progress[i + 1], "i: {}", i);
        }
    }

    #[test]
    fn single_cursor_word_end() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.word_end_default(bs, false);
        };

        let progress = vec![
            "al#a ma kota",
            "ala# ma kota",
            "ala #ma kota",
            "ala ma# kota",
            "ala ma #kota",
            "ala ma kota#",
            "ala ma kota#",
            "ala ma kota#",
        ];

        for i in 0..progress.len() - 1 {
            assert_eq!(apply(progress[i], f), progress[i + 1], "i: {}", i);
        }
    }

    #[test]
    fn multiple_cursors_word_end() {
        let f: fn(&mut CursorSet, &Rope) = |c: &mut CursorSet, bs: &Rope| {
            c.word_end_default(bs, false);
        };

        let progress = vec![
            "ala ma ko#ta\nkot ma# ale\npies sp#i\n#",
            "ala ma kota#\nkot ma #ale\npies spi#\n#",
            "ala ma kota\n#kot ma ale#\npies spi\n#",
            "ala ma kota\nkot# ma ale\n#pies spi\n#",
            "ala ma kota\nkot #ma ale\npies# spi\n#",
            "ala ma kota\nkot ma# ale\npies #spi\n#",
            "ala ma kota\nkot ma #ale\npies spi#\n#",
            "ala ma kota\nkot ma ale#\npies spi\n#",
            "ala ma kota\nkot ma ale\n#pies spi\n#",
            "ala ma kota\nkot ma ale\npies# spi\n#",
            "ala ma kota\nkot ma ale\npies #spi\n#",
            "ala ma kota\nkot ma ale\npies spi#\n#",
            "ala ma kota\nkot ma ale\npies spi\n#",
            "ala ma kota\nkot ma ale\npies spi\n#",
            "ala ma kota\nkot ma ale\npies spi\n#",
        ];

        for i in 0..progress.len() - 1 {
            assert_eq!(apply(progress[i], f), progress[i + 1], "i: {}", i);
        }
    }
}