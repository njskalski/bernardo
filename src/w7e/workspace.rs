use std::path::PathBuf;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::{SerializeSeq, SerializeStruct};
use crate::experiments::pretty_ron::ToPrettyRonString;
use crate::{fs, w7e};
use crate::fs::path::SPath;
use crate::w7e::project_scope;
use crate::w7e::project_scope::{ProjectScope, SerializableProjectScope};

pub const WORKSPACE_FILE_NAME: &'static str = ".gladius_workspace.ron";

pub struct Scopes(Vec<ProjectScope>);
pub type ScopeLoadErrors = Vec<(PathBuf, project_scope::LoadError)>;

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
    pub fn try_load(root_path: SPath) -> Result<(Workspace, ScopeLoadErrors), LoadError> {
        let workspace_file = root_path.descendant_checked(WORKSPACE_FILE_NAME).ok_or(LoadError::WorkspaceFileNotFound)?;
        let serialized_workspace = workspace_file.read_entire_file_to_item::<SerializableWorkspace>()?;
        Self::from(serialized_workspace, root_path)
    }

    pub fn from(sw: SerializableWorkspace, root_path: SPath) -> Result<(Workspace, ScopeLoadErrors), LoadError> {
        let mut scopes: Vec<ProjectScope> = Vec::new();
        let mut scope_errors = ScopeLoadErrors::new();

        for sps in sw.scopes.into_iter() {
            match ProjectScope::from_serializable(sps, &root_path) {
                Ok(scope) => scopes.push(scope),
                Err(error_pair) => scope_errors.push((root_path.relative_path(), error_pair)),
            }
        }

        Ok((Workspace {
            root_path,
            scopes,
        }, scope_errors))
    }
}