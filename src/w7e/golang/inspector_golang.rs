use crate::fs::path::SPath;
use crate::tsw::lang_id::LangId;
use crate::w7e::inspector::LangInspector;

pub struct GoLangInspector {}

impl LangInspector for GoLangInspector {
    fn lang_id(&self) -> LangId {
        LangId::GO
    }

    fn is_project_dir(&self, ff: &SPath) -> bool {
        ff.is_dir() && ff.descendant_checked("go.mod").map(|desc| desc.is_file()).unwrap_or(false)
    }
}

impl GoLangInspector {
    pub fn new() -> Self {
        GoLangInspector {}
    }
}
