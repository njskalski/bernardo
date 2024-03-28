use std::iter::empty;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug)]
pub enum CommonQuery {
    Epsilon,
    String(String),
    Fuzzy(String),
    Regex(regex::Regex),
}

impl CommonQuery {
    pub fn matches(&self, label: &str) -> bool {
        match self {
            CommonQuery::Epsilon => true,
            CommonQuery::String(substr) => label.contains(substr),
            CommonQuery::Fuzzy(subsequence) => {
                let mut text_it = label.graphemes(false).peekable();
                let mut pattern_it = subsequence.graphemes(false).peekable();

                loop {
                    if pattern_it.peek().is_none() {
                        return true;
                    }

                    if text_it.peek().is_none() {
                        return false;
                    }

                    if text_it.peek() == pattern_it.peek() {
                        let _ = text_it.next();
                        let _ = pattern_it.next();
                    } else {
                        text_it.next();
                    }
                }
            }
            CommonQuery::Regex(r) => r.find(label).is_some(),
        }
    }

    pub fn matches_highlights(&self, label: &str) -> Box<dyn Iterator<Item = usize>> {
        match self {
            CommonQuery::Epsilon => Box::new(empty()),
            CommonQuery::String(substr) => match label.find(substr) {
                None => Box::new(empty()),
                Some(pos) => Box::new(pos..pos + substr.graphemes(true).count()),
            },
            CommonQuery::Fuzzy(subsequence) => {
                let mut subsequence_grapheme_it = subsequence.graphemes(true).peekable();
                let label_grapheme_it = label.graphemes(true).enumerate();

                let mut indices: Vec<usize> = Vec::new();

                for (idx, l) in label_grapheme_it {
                    match subsequence_grapheme_it.peek() {
                        None => {
                            return Box::new(indices.into_iter());
                        }
                        Some(s) => {
                            if l == *s {
                                indices.push(idx);
                                subsequence_grapheme_it.next();
                            }
                        }
                    }
                }

                if subsequence_grapheme_it.peek().is_none() {
                    // they ran out at the same time, match
                    return Box::new(indices.into_iter());
                }

                // if I exhausted the label, no match
                Box::new(empty())
            }
            CommonQuery::Regex(_r) => {
                // TODO regex indices?
                Box::new(empty())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy() {
        assert!(CommonQuery::Fuzzy("hell".to_string()).matches("hello"));
        assert_eq!(
            CommonQuery::Fuzzy("hell".to_string())
                .matches_highlights("hello")
                .collect::<Vec<usize>>(),
            vec![0, 1, 2, 3]
        );
        assert!(!CommonQuery::Fuzzy("hell".to_string()).matches("helo"));
    }
}
