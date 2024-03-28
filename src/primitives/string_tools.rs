// pub fn non_consecutive_substring(text: &str, pattern: &str) -> bool {
//     let mut text_it = text.graphemes().peekable();
//     let mut pattern_it = pattern.graphemes().peekable();
//
//     loop {
//         if pattern_it.peek().is_none() {
//             return true;
//         }
//
//         if text_it.peek().is_none() {
//             return false;
//         }
//
//         if text_it.peek() == pattern_it.peek() {
//             let _ = text_it.next();
//             let _ = pattern_it.next();
//             continue;
//         }
//     }
//
//     false
// }
//
// #[cfg(test)]
// mod tests {
//     use crate::primitives::string_tools::non_consecutive_substring;
//
//     #[test]
//     fn non_consecutive_substring_test() {
//         assert_eq!(non_consecutive_substring("abcde", ""), true);
//     }
// }
