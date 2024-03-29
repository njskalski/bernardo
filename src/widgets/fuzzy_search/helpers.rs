use unicode_segmentation::UnicodeSegmentation;

pub fn is_subsequence(item: &str, query: &str) -> bool {
    let mut query_it = query.graphemes(true).peekable();

    for g in item.graphemes(true) {
        if query_it.peek().is_none() {
            break;
        }

        if query_it.peek().map(|f| *f == g).unwrap_or(false) {
            query_it.next();
        }
    }

    query_it.peek().is_none()
}

#[test]
fn test_is_subsequence() {
    assert!(!is_subsequence("abba", "c"));
    assert!(!is_subsequence("abba", "bbb"));
    assert!(is_subsequence("abba", "aba"));
    assert!(is_subsequence("abba", "aa"));
}
