use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LangId {
    BASH,
    C,
    CPP,
    HASKELL,
    GO,
    JAVA,
    JAVASCRIPT,
    PYTHON3,
    RUST,
    TOML,
    TYPESCRIPT,
    HTML,
    YAML,
}

impl LangId {
    pub fn to_lsp_lang_id_string(&self) -> &'static str {
        match self {
            LangId::BASH => "bash",
            LangId::C => "c",
            LangId::CPP => "c++",
            LangId::HASKELL => "haskell",
            LangId::JAVA => "java",
            LangId::JAVASCRIPT => "javascript",
            LangId::GO => "go",
            LangId::PYTHON3 => "python3",
            LangId::RUST => "rust",
            LangId::TOML => "toml",
            LangId::TYPESCRIPT => "typescript",
            LangId::HTML => "html",
            LangId::YAML => "yaml",
        }
    }
}

impl Display for LangId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
