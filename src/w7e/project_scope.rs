use std::any::Any;
use std::path::PathBuf;

use log::error;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::fs::file_front::FileFront;
use crate::w7e::handler::Handler;
use crate::w7e::handler_factory::load_handler;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::LangId;

pub struct ProjectScope {
    pub path: FileFront,
    pub lang_id: LangId,

    /*
    Handler is something that translates "path" to "project definition"
     */
    pub handler: Option<Box<dyn Handler>>,
}

impl Serialize for ProjectScope {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ss = serializer.serialize_struct("ProjectScope", 3)?;
        ss.serialize_field("lang_id", &self.lang_id)?;
        ss.serialize_field("path", &self.path.relative_path())?;
        let handler_id_op = &self.handler.as_ref().map(|h| h.handler_id());
        ss.serialize_field("handler_id", handler_id_op)?;
        ss.end()
    }
}

// impl<'de> Deserialize<'de> for ProjectScope {
//     fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let lang_id = deserializer.deserialize_any::<LangId>()?;
//         let path = deserializer.deserialize_any::<PathBuf>()?;
//
//         let handler_id_op: Option<String> = deserializer.deserialize_any::<Option<String>>()?;
//
//         let handler = handler_id_op
//             .as_ref()
//             .map(|handler_id| match load_handler(handler_id, ()) {
//                 Ok(handler) => Some(handler),
//                 Err(HandlerLoadError::HandlerNotFound) => {
//                     error!("unknown scope handler: \"{}\"", handler_id);
//                     None
//                 }
//                 Err(e) => {
//                     error!("handler load error: \"{}\": {:?}", handler_id, e);
//                     None
//                 }
//             })
//             .flatten();
//     }
// }
