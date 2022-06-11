use regex::{Error, Regex};

#[derive(Debug, Clone)]
pub enum SearchPattern {
    RawString(String),
    Regex(Regex, String),
}

impl SearchPattern {
    pub fn matches(&self, s: &str) -> bool {
        match &self {
            &SearchPattern::RawString(r) => {
                r == s
            }
            SearchPattern::Regex(re, _) => {
                if let Some(m) = re.find_at(s, 0) {
                    m.start() == 0 && m.end() == s.len()
                } else {
                    false
                }
            }
        }
    }
}

impl<T: ToString> From<T> for SearchPattern {
    fn from(t: T) -> Self {
        let s = t.to_string();
        match Regex::new(&s) {
            Ok(re) => SearchPattern::Regex(re, s),
            Err(_) => SearchPattern::RawString(s)
        }
    }
}