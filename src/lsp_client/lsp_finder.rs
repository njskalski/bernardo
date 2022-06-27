use std::path::PathBuf;
use crate::ConfigRef;
use crate::lsp_client::lsp_client::LspWrapper;
use crate::tsw::lang_id::LangId;

/*
This is a class whose job is to provide an initialized LspWrapper or error.
It's job (in future) is to be looking for LSP providers in a reasonable smart manner, with option
to override settings per project.
 */

pub struct LspFinder {
    config: ConfigRef,
}

impl LspFinder {
    pub fn new(config: ConfigRef) -> LspFinder {
        LspFinder {
            config
        }
    }

    pub fn todo_get_lsp(&self, lang_id: LangId, workspace_root: PathBuf) -> Option<LspWrapper> {
        if lang_id == LangId::RUST {
            LspWrapper::todo_new(workspace_root)
        } else {
            None
        }
    }
}
