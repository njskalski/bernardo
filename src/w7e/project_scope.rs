use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{FsfRef, LangId};
use crate::fs::file_front::FileFront;
use crate::w7e::handler::Handler;
use crate::w7e::handler_factory::load_handler;
use crate::w7e::handler_load_error::HandlerLoadError;

pub struct ProjectScope {
    pub lang_id: LangId,
    pub path: FileFront,

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

#[derive(Debug)]
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
            path: self.path.relative_path().to_path_buf(),
            handler_id_op: self.handler.as_ref().map(|h| h.handler_id().to_string()),
        }
    }

    pub fn from_serializable(sps: SerializableProjectScope, fs: FsfRef) -> Result<Self, LoadError> {
        let ff = fs.get_root().descendant(&sps.path).ok_or(LoadError::DirectoryNotFound)?;
        let handler = match sps.handler_id_op {
            None => None,
            Some(handler_id) => {
                Some(load_handler(&handler_id, ff.clone())?)
            }
        };

        Ok(ProjectScope {
            lang_id: sps.lang_id,
            path: ff,
            handler,
        })
    }
}
