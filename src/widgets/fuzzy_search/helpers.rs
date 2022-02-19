use unicode_segmentation::UnicodeSegmentation;

pub fn is_substring(item: &str, query: &str) -> bool {
    let mut query_it = query.graphemes(true).peekable();

    for g in item.graphemes(true) {
        if query_it.peek() == None {
            break;
        }

        if query_it.peek().map(|f| *f == g).unwrap_or(false) {
            query_it.next();
        }
    }

    query_it.peek().is_none()
}

#[test]
fn test_is_substring() {
    assert_eq!(is_substring("abba", "c"), false);
    assert_eq!(is_substring("abba", "bbb"), false);
    assert_eq!(is_substring("abba", "aba"), true);
    assert_eq!(is_substring("abba", "aa"), true);
}