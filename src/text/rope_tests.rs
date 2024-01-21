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

#[test]
fn test_get_line_1() {
    let rope: Box<dyn crate::text::text_buffer::TextBuffer> = Box::new(ropey::Rope::from("aaa\nbbb\n\\ccc"));

    assert_eq!(rope.get_line(0), Some("aaa".to_string()));
    assert_eq!(rope.get_line(1), Some("bbb".to_string()));
    assert_eq!(rope.get_line(2), Some("\\ccc".to_string()));
    assert_eq!(rope.get_line(3), None);
}

#[test]
fn test_get_line_2() {
    let rope: Box<dyn crate::text::text_buffer::TextBuffer> = Box::new(ropey::Rope::from("aaa\nbbb\n\\ccc\n"));

    assert_eq!(rope.get_line(0), Some("aaa".to_string()));
    assert_eq!(rope.get_line(1), Some("bbb".to_string()));
    assert_eq!(rope.get_line(2), Some("\\ccc".to_string()));
    assert_eq!(rope.get_line(3), Some("".to_string()));
}
