use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LangId {
    C,
    CPP,
    HTML,
    GO,
    PYTHON3,
    RUST,
}

impl LangId {
    pub fn to_lsp_lang_id_string(&self) -> &'static str {
        match self {
            LangId::C => "c",
            LangId::CPP => "c++",
            LangId::HTML => "html",
            LangId::GO => "go",
            LangId::PYTHON3 => "python3",
            LangId::RUST => "rust",
        }
    }
}

impl Display for LangId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
