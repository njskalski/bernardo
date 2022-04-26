#[cfg(test)]
mod tests {
    use crate::experiments::clipboard::{Clipboard, ClipboardRef, get_me_fake_clipboard};
    use crate::primitives::common_edit_msgs::{apply_cem, CommonEditMsg};
    use crate::primitives::cursor_set_selection_tests::tests::{buffer_cursors_sel_to_text, text_to_buffer_cursors_with_selections};

    fn text_to_text(text: &str, cem: CommonEditMsg, clipboard: Option<&ClipboardRef>) -> String {
        let (mut buffer, mut cs) = text_to_buffer_cursors_with_selections(text);
        debug_assert!(cs.check_invariants());
        apply_cem(cem, &mut cs, &mut buffer, 4, clipboard);
        debug_assert!(cs.check_invariants());
        buffer_cursors_sel_to_text(&buffer, &cs)
    }

    #[test]
    fn single_cursor_write() {
        assert_eq!(text_to_text("ab#ba", CommonEditMsg::Char('c'), None), "abc#ba");
        assert_eq!(text_to_text("#abba", CommonEditMsg::Char('c'), None), "c#abba");
        assert_eq!(text_to_text("abba#", CommonEditMsg::Char('c'), None), "abbac#");
    }

    #[test]
    fn single_cursor_backspace() {
        assert_eq!(text_to_text("ab#ba", CommonEditMsg::Backspace, None), "a#ba");
        assert_eq!(text_to_text("#abba", CommonEditMsg::Backspace, None), "#abba");
        assert_eq!(text_to_text("abba#", CommonEditMsg::Backspace, None), "abb#");
    }

    #[test]
    fn single_cursor_delete() {
        assert_eq!(text_to_text("ab#da", CommonEditMsg::Delete, None), "ab#a");
        assert_eq!(text_to_text("abda#", CommonEditMsg::Delete, None), "abda#");
        assert_eq!(text_to_text("#abda", CommonEditMsg::Delete, None), "#bda");
    }

    #[test]
    fn multi_cursor_write() {
        assert_eq!(text_to_text("abc#abc#a", CommonEditMsg::Char('d'), None), "abcd#abcd#a");
    }

    #[test]
    fn scenario_1() {
        assert_eq!(text_to_text("#\n#\n#\n#\n", CommonEditMsg::Char('a'), None), "a#\na#\na#\na#\n");
        assert_eq!(text_to_text("a#\na#\na#\na#\n", CommonEditMsg::Char('b'), None), "ab#\nab#\nab#\nab#\n");
        assert_eq!(text_to_text("ab#\nab#\nab#\nab#\n", CommonEditMsg::CursorLeft { selecting: true }, None), "a[b)\na[b)\na[b)\na[b)\n");
        assert_eq!(text_to_text("a[b)\na[b)\na[b)\na[b)\n", CommonEditMsg::Char('x'), None), "ax#\nax#\nax#\nax#\n");
        assert_eq!(text_to_text("ax#\nax#\nax#\nax#\n", CommonEditMsg::WordBegin { selecting: true }, None), "[ax)\n[ax)\n[ax)\n[ax)\n");
        assert_eq!(text_to_text("[ax)\n[ax)\n[ax)\n[ax)\n", CommonEditMsg::Char('u'), None), "u#\nu#\nu#\nu#\n");
        assert_eq!(text_to_text("u#\nu#\nu#\nu#\n", CommonEditMsg::Backspace, None), "#\n#\n#\n#\n");
        assert_eq!(text_to_text("#\n#\n#\n#\n", CommonEditMsg::Backspace, None), "#\n");
    }

    #[test]
    fn multi_cursor_backspace() {
        assert_eq!(text_to_text("#\n#\n#\n#\n", CommonEditMsg::Backspace, None), "#\n");
    }

    #[test]
    fn multi_cursor_delete() {
        assert_eq!(text_to_text("#ab#ab#ab#ab", CommonEditMsg::Delete, None), "#b#b#b#b");
        assert_eq!(text_to_text("#\n#\n#\n#\n", CommonEditMsg::Delete, None), "#");
    }

    #[test]
    fn multi_cursor_copy_paste() {
        let clipboard = get_me_fake_clipboard();
        let c = Some(&clipboard);

        assert_eq!(text_to_text("#abba\n#abba\n#abba\n#abba\n", CommonEditMsg::CursorRight { selecting: true }, c), "(a]bba\n(a]bba\n(a]bba\n(a]bba\n");
        assert_eq!(text_to_text("(a]bba\n(a]bba\n(a]bba\n(a]bba\n", CommonEditMsg::CursorRight { selecting: true }, c), "(ab]ba\n(ab]ba\n(ab]ba\n(ab]ba\n");
        assert_eq!(text_to_text("(ab]ba\n(ab]ba\n(ab]ba\n(ab]ba\n", CommonEditMsg::Copy, c), "(ab]ba\n(ab]ba\n(ab]ba\n(ab]ba\n");
        assert_eq!(text_to_text("(ab]ba\n(ab]ba\n(ab]ba\n(ab]ba\n", CommonEditMsg::LineEnd { selecting: false }, c), "abba#\nabba#\nabba#\nabba#\n");
        assert_eq!(text_to_text("abba#\nabba#\nabba#\nabba#\n", CommonEditMsg::Paste, c), "abbaab#\nabbaab#\nabbaab#\nabbaab#\n");
    }
}