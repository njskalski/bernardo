use std::any::Any;
use std::fmt;
use std::path::PathBuf;

use log::error;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{DeserializeSeed, EnumAccess, Error, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, SerializeStruct, SerializeTuple};

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

impl Serialize for ProjectScope {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer {
        let mut ss = serializer.serialize_struct("ProjectScope", 3)?;
        ss.serialize_field("lang_id", &self.lang_id)?;
        ss.serialize_field("path", &self.path.relative_path())?;
        let handler_id_op = &self.handler.as_ref().map(|h| h.handler_id());
        ss.serialize_field("handler", handler_id_op)?;
        ss.end()
    }
}

// https://serde.rs/deserialize-struct.html
impl<'de> DeserializeSeed<'de> for FsfRef {
    type Value = ProjectScope;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>, {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field { LangId, Path, Handler }

        struct FsfRefVisitor(FsfRef);

        impl<'de> Visitor<'de> for FsfRefVisitor {
            type Value = ProjectScope;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "struct ProjectScope")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ProjectScope, V::Error>
                where
                    V: MapAccess<'de>,
            {
                let mut lang_id: Option<LangId> = None;
                let mut path: Option<PathBuf> = None;
                let mut handler_id_op: Option<String> = None;

                while let Some(key) = map.next_key_seed(&self)? {
                    match key {
                        Field::LangId => {
                            if lang_id.is_some() {
                                return Err(de::Error::duplicate_field("lang_id"));
                            }
                            lang_id = Some(map.next_value()?);
                        }
                        Field::Path => {
                            if path.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                        }
                        Field::Handler => {
                            if handler_id_op.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            handler_id_op = Some(map.next_value()?);
                        }
                    }
                }

                let handler = handler_id_op.map(|id| load_handler(&id).unwrap());

                Ok(ProjectScope {
                    lang_id: LangId::C,
                    path: self.0.get_root(),
                    handler,
                })
            }
        }

        deserializer.deserialize_struct("ProjectScope",
                                        &["lang_id", "path", "handler"],
                                        FsfRefVisitor(self))
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
