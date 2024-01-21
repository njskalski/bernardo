use crate::fs::path::SPath;
use crate::tsw::lang_id::LangId;
use crate::w7e::inspector::LangInspector;

pub struct CppLangInspector {}

impl LangInspector for CppLangInspector {
    fn lang_id(&self) -> LangId {
        LangId::CPP
    }

    fn is_project_dir(&self, ff: &SPath) -> bool {
        ff.is_dir()
            && ff
                .descendant_checked("compile_commands.json")
                .map(|desc| desc.is_file())
                .unwrap_or(false)
    }
}

impl CppLangInspector {
    pub fn new() -> Self {
        CppLangInspector {}
    }
}
