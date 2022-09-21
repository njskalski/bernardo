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
            CommonQuery::Epsilon => {
                true
            }
            CommonQuery::String(substr) => {
                label.contains(substr)
            }
            CommonQuery::Fuzzy(subsequence) => {
                let mut subsequence_grapheme_it = subsequence.graphemes(true).peekable();
                let mut label_grapheme_it = label.graphemes(true);

                while let Some(l) = label_grapheme_it.next() {
                    match subsequence_grapheme_it.peek() {
                        None => {
                            return true;
                        }
                        Some(s) => {
                            if l == *s {
                                subsequence_grapheme_it.next();
                            }
                        }
                    }
                }

                if subsequence_grapheme_it.peek().is_none() {
                    // they ran out at the same time, match
                    return true;
                }

                // if I exhausted the label, no match
                false
            }
            CommonQuery::Regex(r) => {
                r.find(label).is_some()
            }
        }
    }

    pub fn matches_highlights(&self, label: &str) -> Box<dyn Iterator<Item=usize>> {
        match self {
            CommonQuery::Epsilon => {
                Box::new(empty())
            }
            CommonQuery::String(substr) => {
                match label.find(substr) {
                    None => Box::new(empty()),
                    Some(pos) => {
                        Box::new((pos..pos + substr.graphemes(true).count()).into_iter())
                    }
                }
            }
            CommonQuery::Fuzzy(subsequence) => {
                let mut subsequence_grapheme_it = subsequence.graphemes(true).peekable();
                let mut label_grapheme_it = label.graphemes(true).enumerate();

                let mut indices: Vec<usize> = Vec::new();

                while let Some((idx, l)) = label_grapheme_it.next() {
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
            CommonQuery::Regex(r) => {
                // TODO regex indices?
                Box::new(empty())
            }
        }
    }
}