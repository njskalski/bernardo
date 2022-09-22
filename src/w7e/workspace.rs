use std::path::PathBuf;
use std::sync::Arc;

use log::debug;
use serde::{Deserialize, Serialize};

use crate::config::config::ConfigRef;
use crate::experiments::pretty_ron::ToPrettyRonString;
use crate::fs;
use crate::fs::path::SPath;
use crate::fs::write_error::WriteOrSerError;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_group::{NavCompGroup, NavCompGroupRef};
use crate::w7e::project_scope;
use crate::w7e::project_scope::{ProjectScope, SerializableProjectScope};

/*
So a funny finding while adding NavCompTick channel is that it seems like I could immediately add
a parallel channel for "filesystem events" within the workspace (after all we don't care for other),
or even go so far as to merge the two, NavCompTicks being just a branch of multiple ticks we'd like
to receive.

In a way these two not obviously related structures seem to be intertwined.
 */

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
    ReadError(fs::read_error::ReadError),
}

impl From<fs::read_error::ReadError> for LoadError {
    fn from(re: fs::read_error::ReadError) -> Self {
        LoadError::ReadError(re)
    }
}

impl Workspace {
    pub fn new(root_path: SPath, scopes: Vec<ProjectScope>) -> Workspace {
        Workspace {
            root_path,
            scopes,
        }
    }

    pub fn try_load(root_path: SPath) -> Result<(Workspace, ScopeLoadErrors), LoadError> {
        let workspace_file = root_path.descendant_checked(WORKSPACE_FILE_NAME).ok_or(LoadError::WorkspaceFileNotFound)?;
        debug!("loading workspace file from {:?}", workspace_file.absolute_path());
        let serialized_workspace = workspace_file.read_entire_file_to_item::<SerializableWorkspace>()?;
        Self::from(serialized_workspace, root_path)
    }

    pub fn save(&self) -> Result<usize, WriteOrSerError> {
        let file = self.root_path.descendant_unchecked(WORKSPACE_FILE_NAME).unwrap();
        let pill = self.serializable();
        file.overwrite_with_ron(&pill)
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

    pub fn serializable(&self) -> SerializableWorkspace {
        let serializable_scopes: Vec<_> = self.scopes.iter().map(|scope| scope.serializable()).collect();
        SerializableWorkspace {
            scopes: serializable_scopes
        }
    }

    pub fn initialize_handlers(&mut self, config: &ConfigRef) -> (NavCompGroupRef, Vec<HandlerLoadError>) {
        let mut errors: Vec<HandlerLoadError> = Vec::default();
        let mut nav_comp_group = NavCompGroup::new();

        for scope in self.scopes.iter_mut() {
            match scope.load_handler(config,
                                     nav_comp_group.todo_sender().clone()) {
                Ok(_) => {
                    let mut has_navcomp = false;
                    scope.handler.as_ref().map(|h| {
                        if let Some(navcomp) = h.navcomp() {
                            has_navcomp = true;
                            nav_comp_group.add_option(h.lang_id(), navcomp);
                        }
                    });

                    debug!("loaded handler for scope {:?}, has_navcomp: {}", scope.path.absolute_path(), has_navcomp);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        debug!("created navcomp group with {} items", nav_comp_group.len());

        (Arc::new(nav_comp_group), errors)
    }

    pub fn scopes(&self) -> &Vec<ProjectScope> {
        &self.scopes
    }
}