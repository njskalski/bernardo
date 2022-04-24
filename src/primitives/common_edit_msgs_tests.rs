#[cfg(test)]
mod tests {
    use crate::primitives::common_edit_msgs::{apply_cem, CommonEditMsg};
    use crate::primitives::cursor_set_selection_tests::tests::{buffer_cursors_sel_to_text, text_to_buffer_cursors_with_selections};

    fn text_to_text(text: &str, cem: CommonEditMsg) -> String {
        let (mut buffer, mut cs) = text_to_buffer_cursors_with_selections(text);
        debug_assert!(cs.check_invariants());
        apply_cem(cem, &mut cs, &mut buffer, 4, None);
        debug_assert!(cs.check_invariants());
        buffer_cursors_sel_to_text(&buffer, &cs)
    }

    #[test]
    fn single_cursor_write() {
        assert_eq!(text_to_text("ab#ba", CommonEditMsg::Char('c')), "abc#ba");
        assert_eq!(text_to_text("#abba", CommonEditMsg::Char('c')), "c#abba");
        assert_eq!(text_to_text("abba#", CommonEditMsg::Char('c')), "abbac#");
    }

    #[test]
    fn single_cursor_backspace() {
        assert_eq!(text_to_text("ab#ba", CommonEditMsg::Backspace), "a#ba");
        assert_eq!(text_to_text("#abba", CommonEditMsg::Backspace), "#abba");
        assert_eq!(text_to_text("abba#", CommonEditMsg::Backspace), "abb#");
    }

    #[test]
    fn single_cursor_delete() {
        assert_eq!(text_to_text("ab#da", CommonEditMsg::Delete), "ab#a");
        assert_eq!(text_to_text("abda#", CommonEditMsg::Delete), "abda#");
        assert_eq!(text_to_text("#abda", CommonEditMsg::Delete), "#bda");
    }

    #[test]
    fn multi_cursor_write() {
        assert_eq!(text_to_text("abc#abc#a", CommonEditMsg::Char('d')), "abcd#abcd#a");
    }

    #[test]
    fn scenario_1() {
        assert_eq!(text_to_text("#\n#\n#\n#\n", CommonEditMsg::Char('a')), "a#\na#\na#\na#\n");
        assert_eq!(text_to_text("a#\na#\na#\na#\n", CommonEditMsg::Char('b')), "ab#\nab#\nab#\nab#\n");
        assert_eq!(text_to_text("ab#\nab#\nab#\nab#\n", CommonEditMsg::CursorLeft { selecting: true }), "a[b)\na[b)\na[b)\na[b)\n");
        assert_eq!(text_to_text("a[b)\na[b)\na[b)\na[b)\n", CommonEditMsg::Char('x')), "ax#\nax#\nax#\nax#\n");
        assert_eq!(text_to_text("ax#\nax#\nax#\nax#\n", CommonEditMsg::WordBegin { selecting: true }), "[ax)\n[ax)\n[ax)\n[ax)\n");
        assert_eq!(text_to_text("[ax)\n[ax)\n[ax)\n[ax)\n", CommonEditMsg::Char('u')), "u#\nu#\nu#\nu#\n");
        assert_eq!(text_to_text("u#\nu#\nu#\nu#\n", CommonEditMsg::Backspace), "#\n#\n#\n#\n");
        assert_eq!(text_to_text("#\n#\n#\n#\n", CommonEditMsg::Backspace), "#\n");
    }

    #[test]
    fn multi_cursor_backspace() {
        assert_eq!(text_to_text("#\n#\n#\n#\n", CommonEditMsg::Backspace), "#\n");
    }

    #[test]
    fn multi_cursor_delete() {
        assert_eq!(text_to_text("#ab#ab#ab#ab", CommonEditMsg::Delete), "#b#b#b#b");
        assert_eq!(text_to_text("#\n#\n#\n#\n", CommonEditMsg::Delete), "#");
    }
}