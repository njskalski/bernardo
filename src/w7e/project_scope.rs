use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::LangId;
use crate::experiments::pretty_ron::ToPrettyRonString;
use crate::new_fs::fsf_ref::FsfRef;
use crate::new_fs::path::SPath;
use crate::w7e::handler::Handler;
use crate::w7e::handler_factory::load_handler;
use crate::w7e::handler_load_error::HandlerLoadError;

pub struct ProjectScope {
    pub lang_id: LangId,
    pub path: SPath,

    /*
    Handler is something that translates "path" to "project definition"
     */
    pub handler: Option<Box<dyn Handler>>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializableProjectScope {
    pub lang_id: LangId,
    pub path: PathBuf,
    pub handler_id_op: Option<String>,
}

impl ToPrettyRonString for SerializableProjectScope {}

#[derive(Debug, Eq, PartialEq)]
pub enum LoadError {
    DirectoryNotFound,
    HandlerLoadError(HandlerLoadError),
}

impl From<HandlerLoadError> for LoadError {
    fn from(e: HandlerLoadError) -> Self {
        LoadError::HandlerLoadError(e)
    }
}

impl ProjectScope {
    pub fn serializable(&self) -> SerializableProjectScope {
        SerializableProjectScope {
            lang_id: self.lang_id,
            path: self.path.relative_path(),
            handler_id_op: self.handler.as_ref().map(|h| h.handler_id().to_string()),
        }
    }

    pub fn from_serializable(sps: SerializableProjectScope, workspace : &SPath) -> Result<Self, LoadError> {
        let ff = workspace.descendant_checked(&sps.path).ok_or(LoadError::DirectoryNotFound)?;
        let handler = match &sps.handler_id_op {
            None => None,
            Some(handler_id) => {
                match load_handler(&handler_id, ff.clone()) {
                    Ok(s) => Some(s),
                    Err(e) => {
                        return Err(LoadError::HandlerLoadError(e));
                    }
                }
            }
        };

        Ok(ProjectScope {
            lang_id: sps.lang_id,
            path: ff,
            handler,
        })
    }
}
