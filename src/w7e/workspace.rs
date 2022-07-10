use std::path::PathBuf;

use serde::Serialize;

use crate::fs::file_front::FileFront;
use crate::fs::filesystem_front::FilesystemFront;
use crate::w7e::handler::Handler;
use crate::w7e::project_scope::ProjectScope;
use crate::LangId;

// use crate::w7e::inspector::Inspector;

// #[derive(Serialize, Debug, Deserialize)]
pub struct Workspace {
    root_path: FileFront,
    scopes: Vec<ProjectScope>,
}
