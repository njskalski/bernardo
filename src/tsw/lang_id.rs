#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LangId {
    C,
    CPP,
    HTML,
    ELM,
    GO,
    RUST,
}

impl LangId {
    pub fn to_lsp_lang_id_string(&self) -> &'static str {
        match self {
            LangId::C => "c",
            LangId::CPP => "c++",
            LangId::HTML => "html",
            LangId::ELM => "elm",
            LangId::GO => "go",
            LangId::RUST => "rust",
        }
    }
}