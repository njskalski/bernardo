use std::path::PathBuf;

use log::{debug, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{ConfigRef, LangId};
use crate::experiments::pretty_ron::ToPrettyRonString;
use crate::fs::path::SPath;
use crate::w7e::handler::Handler;
use crate::w7e::handler_factory::load_handler;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_group::{NavCompTick, NavCompTickSender};

pub struct ProjectScope {
    pub lang_id: LangId,
    pub path: SPath,

    /*
    Handler is something that translates "path" to "project definition"
     */
    pub handler_id: Option<String>,
    pub handler: Option<Box<dyn Handler>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableProjectScope {
    pub lang_id: LangId,
    pub path: PathBuf,
    pub handler_id_op: Option<String>,
}

impl ToPrettyRonString for SerializableProjectScope {}

#[derive(Debug, Eq, PartialEq)]
pub enum LoadError {
    DirectoryNotFound,
}

impl ProjectScope {
    pub fn serializable(&self) -> SerializableProjectScope {
        SerializableProjectScope {
            lang_id: self.lang_id,
            path: self.path.relative_path(),
            handler_id_op: self.handler.as_ref().map(|h| h.handler_id().to_string()),
        }
    }

    pub fn from_serializable(sps: SerializableProjectScope, workspace: &SPath) -> Result<Self, LoadError> {
        debug!("loading project scope from pill: {:?}", sps);
        let ff = if sps.path.as_os_str().is_empty() {
            workspace.clone()
        } else {
            workspace.descendant_checked(&sps.path).ok_or(LoadError::DirectoryNotFound)?
        };

        Ok(ProjectScope {
            lang_id: sps.lang_id,
            path: ff,
            handler_id: sps.handler_id_op,
            handler: None,
        })
    }

    /*
    Config is required to "know" where the LSP servers are. We will provide reasonable defaults,
    but option to override is essential.
     */
    pub fn load_handler(&mut self,
                        config: &ConfigRef,
                        navcomp_tick_sender: NavCompTickSender,
    ) -> Result<(), HandlerLoadError> {
        let handler = match &self.handler_id {
            None => {
                warn!("project scope [{:?}] with no handler - what the point?", self.path.relative_path());
                return Ok(());
            }
            Some(handler_id) => {
                load_handler(config,
                             &handler_id,
                             self.path.clone(),
                             navcomp_tick_sender.clone(),
                )?
            }
        };

        self.handler = Some(handler);
        Ok(())
    }
}
