#[cfg(test)]
mod tests {
    use crate::primitives::common_edit_msgs::{apply_cem, CommonEditMsg};
    use crate::primitives::cursor_set::CursorSet;
    use crate::primitives::cursor_set_selection_tests::tests::{buffer_cursors_sel_to_text, text_to_buffer_cursors_with_selections};
    use crate::text::buffer::Buffer;

    fn text_to_text(text: &str, cem: CommonEditMsg) -> String {
        let (mut buffer, mut cs) = text_to_buffer_cursors_with_selections(text);
        debug_assert!(cs.check_invariants());
        apply_cem(cem, &mut cs, &mut buffer, 4);
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
        assert_eq!(text_to_text("abc#abc#", CommonEditMsg::Char('d')), "abcd#abcd#");
    }

    #[test]
    fn scenario_1() {
        assert_eq!(text_to_text("#\n#\n#\n#\n", CommonEditMsg::Char('a')), "a#\na#\na#\na#\n");
        assert_eq!(text_to_text("a#\na#\na#\na#\n", CommonEditMsg::Char('b')), "ab#\nab#\nab#\nab#\n");
        assert_eq!(text_to_text("ab#\nab#\nab#\nab#\n", CommonEditMsg::CursorLeft { selecting: true }), "a#b]\na#b]\na#b]\na#b]\n");
    }

    #[test]
    fn scenario_1_1() {
        assert_eq!(text_to_text("a#b]\na#b]\na#b]\na#b]\n", CommonEditMsg::Char('x')), "ax#\nax#\nax#\nax#\n");
    }
}