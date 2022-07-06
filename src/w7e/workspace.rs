use std::path::PathBuf;
use crate::fs::file_front::FileFront;
use crate::fs::filesystem_front::FilesystemFront;
use crate::LangId;
use crate::w7e::handler::Handler;
// use crate::w7e::inspector::Inspector;

pub struct ProjectScope {
    pub path: FileFront,
    pub lang_id: LangId,

    /*
    Handler is something that translates "path" to "project definition"
     */
    pub handler: Option<Box<dyn Handler>>,
}

// #[derive(Serialize, Debug, Deserialize)]
pub struct Workspace {
    root_path: FileFront,
    scopes: Vec<ProjectScope>,
}
