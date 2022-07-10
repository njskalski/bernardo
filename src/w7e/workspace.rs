use crate::fs::file_front::FileFront;
use crate::w7e::project_scope::ProjectScope;

// use crate::w7e::inspector::Inspector;

// #[derive(Serialize, Debug, Deserialize)]
pub struct Workspace {
    root_path: FileFront,
    scopes: Vec<ProjectScope>,
}
