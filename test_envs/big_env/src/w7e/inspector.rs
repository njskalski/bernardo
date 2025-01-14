use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use lazy_static::lazy_static;
use log::debug;
use maplit::hashmap;

use crate::fs::path::SPath;
use crate::tsw::lang_id::LangId;
use crate::w7e::project_scope::ProjectScope;
use crate::w7e::rust::inspector_rust::RustLangInspector;

#[derive(Debug)]
pub enum InspectError {
    NotAFolder,
}

impl Display for InspectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //TODO
        write!(f, "{:?}", self)
    }
}

pub trait LangInspector: Sync {
    fn lang_id(&self) -> LangId;

    /*
    This is supposed to be quick.
     */
    fn is_project_dir(&self, ff: &SPath) -> bool;
}

lazy_static! {
    static ref KNOWN_INSPECTORS: HashMap<LangId, Box<dyn LangInspector>> = hashmap! {
        LangId::RUST => Box::new(RustLangInspector::new()) as Box<dyn LangInspector>,
    };
}

/*
This is a stub method that is supposed to figure out if there are projects to be found in this
directory.
 */
pub fn inspect_workspace(folder: &SPath) -> Result<Vec<ProjectScope>, InspectError> {
    if !folder.is_dir() {
        return Err(InspectError::NotAFolder);
    }

    let mut scopes: Vec<ProjectScope> = Vec::new();

    // TODO add one level more of descending (multiple projects per dir)

    for (lang_id, inspector) in KNOWN_INSPECTORS.iter() {
        debug!("checking for lang {} in {}", lang_id, &folder);
        if inspector.is_project_dir(&folder) {
            debug!("matched {}", lang_id);
            scopes.push(ProjectScope {
                path: folder.clone(),
                lang_id: inspector.lang_id(),
                // this is a place where we set default handler_ids
                handler_id: Some(inspector.lang_id().to_lsp_lang_id_string().to_string()),
                handler: None,
            });
        }
    }

    Ok(scopes)
}
