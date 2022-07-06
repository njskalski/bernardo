// use crate::fs::file_front::FileFront;
// use crate::LangId;
// use crate::w7e::handler_load_error::HandlerLoadError;
// use crate::w7e::inspector::Inspector;
//
// pub struct InspectorRust {
//     root: FileFront
// }
//
// impl Inspector for InspectorRust {
//     fn lang_id() -> LangId {
//         LangId::RUST
//     }
// }
//
// impl InspectorRust {
//     fn load(ff: FileFront) -> Result<Self, HandlerLoadError> {
//         Ok(InspectorRust {
//             root: ff
//         })
//     }
// }