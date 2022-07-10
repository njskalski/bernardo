use serde::{Serialize, Serializer};

use crate::fs::file_front::FileFront;
use crate::w7e::handler::Handler;
use crate::LangId;

pub struct ProjectScope {
    pub path: FileFront,
    pub lang_id: LangId,

    /*
    Handler is something that translates "path" to "project definition"
     */
    pub handler: Option<Box<dyn Handler>>,
}

// impl Serialize for ProjectScope {
//     fn serialize<S>(&self, serializer: S) -> Result<serde::ser::Ok, serde::ser::Error>
//     where
//         S: Serializer,
//     {
//         serializer.ser
//     }
// }
