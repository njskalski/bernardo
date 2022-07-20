use std::path::PathBuf;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::{SerializeSeq, SerializeStruct};
use crate::experiments::pretty_ron::ToPrettyRonString;
use crate::{new_fs, w7e};
use crate::new_fs::path::SPath;
use crate::w7e::project_scope::{ProjectScope, SerializableProjectScope};

pub const WORKSPACE_FILE_NAME: &'static str = ".gladius_workspace.ron";

pub struct Scopes(Vec<ProjectScope>);

pub struct Workspace {
    root_path: SPath,
    scopes: Vec<ProjectScope>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializableWorkspace {
    pub scopes: Vec<SerializableProjectScope>,
}

impl ToPrettyRonString for SerializableWorkspace {}

#[derive(Debug)]
pub enum LoadError {
    WorkspaceFileNotFound,
    ScopeLoadError(w7e::project_scope::LoadError),
    ReadError(new_fs::read_error::ReadError),
}

impl From<w7e::project_scope::LoadError> for LoadError {
    fn from(e: w7e::project_scope::LoadError) -> Self {
        LoadError::ScopeLoadError(e)
    }
}

impl From<new_fs::read_error::ReadError> for LoadError {
    fn from(re: new_fs::read_error::ReadError) -> Self {
        LoadError::ReadError(re)
    }
}

impl Workspace {
    pub fn try_load(root_path: SPath) -> Result<Workspace, LoadError> {
        let x = root_path.relative_path();

        let workspace_file = root_path.descendant_checked(WORKSPACE_FILE_NAME).ok_or(LoadError::WorkspaceFileNotFound)?;
        let serialized_workspace = workspace_file.read_entire_file_to_item::<SerializableWorkspace>()?;
        let workspace = Self::from(serialized_workspace, root_path)?;
        Ok(workspace)
    }

    pub fn from(sw: SerializableWorkspace, root_path: SPath) -> Result<Workspace, LoadError> {
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