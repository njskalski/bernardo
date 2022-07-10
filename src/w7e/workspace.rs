use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::{SerializeSeq, SerializeStruct};

use crate::{fs, w7e};
use crate::fs::file_front::FileFront;
use crate::w7e::project_scope::{ProjectScope, SerializableProjectScope};

pub const WORKSPACE_FILE: &'static str = ".gladius_workspace.ron";

pub struct Scopes(Vec<ProjectScope>);

pub struct Workspace {
    root_path: FileFront,
    scopes: Vec<ProjectScope>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializableWorkspace {
    scopes: Vec<SerializableProjectScope>,
}

#[derive(Debug)]
pub enum LoadError {
    WorkspaceFileNotFound,
    ScopeLoadError(w7e::project_scope::LoadError),
    ReadError(fs::read_error::ReadError),
}

impl From<w7e::project_scope::LoadError> for LoadError {
    fn from(e: w7e::project_scope::LoadError) -> Self {
        LoadError::ScopeLoadError(e)
    }
}

impl From<fs::read_error::ReadError> for LoadError {
    fn from(re: fs::read_error::ReadError) -> Self {
        LoadError::ReadError(re)
    }
}

impl Workspace {
    pub fn try_load(root_path: FileFront) -> Result<Workspace, LoadError> {
        let workspace_file = root_path.descendant(WORKSPACE_FILE).ok_or(LoadError::WorkspaceFileNotFound)?;
        let serialized_workspace = workspace_file.read_entire_file_to_bytes()
    }

    pub fn from(sw: SerializableWorkspace, root_path: FileFront) -> Result<Workspace, LoadError> {
        let mut scopes: Vec<ProjectScope> = Vec::new();
        for sps in sw.scopes {
            let item = ProjectScope::from_serializable(sps, root_path.fsf().clone())?;
            scopes.push(item);
        }

        Ok(Workspace {
            root_path,
            scopes,
        })
    }
}