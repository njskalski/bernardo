// This file exists for two reasons. First, so I am sure I understand rope. Second, that the
// contracts will not change silently on some update.

#[test]
fn rope_last_line_newline() {
    let rope = ropey::Rope::from("aaa\nbbb\n");

    assert_eq!(3, rope.len_lines());
    assert_eq!(8, rope.len_chars());
    assert_eq!(1, rope.try_char_to_line(7).unwrap());
    assert_eq!(2, rope.try_char_to_line(8).unwrap());
    assert_eq!(true, rope.try_char_to_line(9).is_err());

    assert_eq!(8, rope.line_to_char(2));
}