#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct LanguageSet {
    pub bash: bool,
    pub c: bool,
    pub cpp: bool,
    pub go: bool,
    pub haskell: bool,
    pub html: bool,
    pub java: bool,
    pub javascript: bool,
    pub python3: bool,
    pub rust: bool,
    pub toml : bool,
    pub typescript: bool,
}

impl LanguageSet {
    pub fn full() -> Self {
        LanguageSet {
            bash: true,
            c: true,
            cpp: true,
            go: true,
            haskell: true,
            html: true,
            java: true,
            javascript: true,
            python3: true,
            rust: true,
            toml: true,
            typescript: true,
        }
    }
}
