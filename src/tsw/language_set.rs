#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct LanguageSet {
    pub c: bool,
    pub cpp: bool,
    pub go: bool,
    pub html: bool,
    pub python3: bool,
    pub rust: bool,
}

impl LanguageSet {
    pub fn full() -> Self {
        LanguageSet {
            c: true,
            cpp: true,
            go: true,
            html: true,
            python3: true,
            rust: true,
        }
    }
}
