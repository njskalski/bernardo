use crate::fs::file_front::FileFront;
use crate::LangId;
use crate::w7e::workspace::ProjectScope;
use std::collections::HashMap;
use log::error;
use crate::w7e::handler::Handler;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::rust::inspector_rust::RustLangInspector;

#[derive(Debug)]
pub enum InspectError {
    NotAFolder,
}

pub trait LangInspector: Sync {
    fn lang_id(&self) -> LangId;

    /*
    This is supposed to be quick.
     */
    fn is_project_dir(&self, ff: &FileFront) -> bool;

    fn handle(&self, ff: FileFront) -> Result<Box<dyn Handler>, HandlerLoadError>;
}

lazy_static! {
    static ref KnownInspectors : HashMap<LangId, Box<dyn LangInspector>> = hashmap! {
        LangId::RUST => Box::new(RustLangInspector::new()) as Box<dyn LangInspector>,
    };
}

/*
This is a stub method that is supposed to figure out if there are projects to be found in this
directory.
 */
pub fn inspect_workspace(folder: FileFront) -> Result<Vec<ProjectScope>, InspectError> {
    if !folder.is_dir() {
        return Err(InspectError::NotAFolder);
    }

    let mut scopes: Vec<ProjectScope> = Vec::new();

    // TODO add one level more of descending (multiple projects per dir)

    for (lang_id, inspector) in KnownInspectors.iter() {
        if inspector.is_project_dir(&folder) {
            match inspector.handle(folder.clone()) {
                Ok(handler) => {
                    scopes.push(ProjectScope {
                        path: folder.clone(),
                        lang_id: inspector.lang_id(),
                        handler: Some(handler),
                    })
                }
                Err(e) => {
                    error!("handler {} failed: {:?}", lang_id, e);
                }
            }
        }
    }

    Ok(scopes)
}