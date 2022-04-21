#[cfg(test)]
mod tests {
    use crate::primitives::common_edit_msgs::{apply_cem, CommonEditMsg};
    use crate::primitives::cursor_set_tests::tests::{buffer_cursors_to_text, text_to_buffer_cursors};

    fn text_to_text(text: &str, cem: CommonEditMsg) -> String {
        let (mut buffer, mut cursors) = text_to_buffer_cursors(text);
        debug_assert!(cursors.check_invariants());
        apply_cem(cem, &mut cursors, &mut buffer, 4);
        buffer_cursors_to_text(&buffer, &cursors)
    }

    #[test]
    fn single_cursor_write() {
        assert_eq!(text_to_text("ab#ba", CommonEditMsg::Char('c')), "abc#ba");
    }

    #[test]
    fn single_cursor_backspace() {
        assert_eq!(text_to_text("ab#ba", CommonEditMsg::Backspace), "a#ba");
    }

    #[test]
    fn single_cursor_delete() {
        assert_eq!(text_to_text("ab#da", CommonEditMsg::Delete), "ab#a");
    }
}