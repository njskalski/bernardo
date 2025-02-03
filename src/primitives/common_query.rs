use log::error;
use regex::Regex;
use std::iter::empty;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug)]
pub enum CommonQuery {
    Epsilon,
    String(String),
    Fuzzy(String),
    Regex(regex::Regex),
}

// (task 79)
// Warning: currently, CommonQuery matches *characters* not *graphemes*
// this might cause errors in situations, where a split-multi-character-grapheme pattern
// is false-positively matched with a label that contains all characters, but not the grapheme
// itself. To address this, and re-introduce per-grapheme matches, we need to introduce
// grapheme-division-caching (special variant of strings), because current "graphemes"
// implementation is too slow.

impl CommonQuery {
    pub fn matches(&self, label: &str) -> bool {
        match self {
            CommonQuery::Epsilon => true,
            CommonQuery::String(substr) => label.contains(substr),
            CommonQuery::Fuzzy(subsequence) => {
                let mut text_it = label.chars();
                let mut pattern_it = subsequence.chars();

                let mut current_pattern_value = pattern_it.next();
                let mut current_text_value = text_it.next();

                loop {
                    if current_pattern_value.is_none() {
                        return true;
                    }

                    if current_text_value.is_none() {
                        return false;
                    }

                    if current_text_value == current_pattern_value {
                        current_text_value = text_it.next();
                        current_pattern_value = pattern_it.next();
                    } else {
                        current_text_value = text_it.next();
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
                error!("highlighting of regex matching indices not implemented yet, returning empty iterator");
                // TODO regex indices?
                Box::new(empty())
            }
        }
    }

    pub fn new_interpreting_wildcards(pattern: &str) -> Option<CommonQuery> {
        if pattern.contains("*") || pattern.contains("?") {
            let regex_pattern = format!("^{}$", pattern.replace(".", r"\.").replace("*", ".*").replace("?", "."));

            let regex = Regex::new(&regex_pattern).ok()?;

            Some(CommonQuery::Regex(regex))
        } else {
            match pattern {
                "" => Some(CommonQuery::Epsilon),
                _ => Some(CommonQuery::String(pattern.to_string())),
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
