use crate::new_fs::path::SPath;
use crate::w7e::handler::Handler;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::rust::handler_rust::RustHandler;

// pub fn load_handler(
//     handler_id: &str,
// ) -> Option<fn(ff) -> Result<Box<dyn Handler>, HandlerLoadError>> {
//     match handler_id {
//         "rust" => Some(|ff| RustHandler::load(ff).map(|h| Box::new(h) as Box<dyn Handler>)),
//         _ => None,
//     }
// }

pub fn load_handler(handler_id: &str, ff: SPath) -> Result<Box<dyn Handler>, HandlerLoadError> {
    match handler_id {
        "rust" => {
            // RustHandler::load(ff).map(|handler| Ok(Box::new(handler) as Box<dyn Handler>))
            match RustHandler::load(ff) {
                Ok(o) => Ok(Box::new(o)),
                Err(e) => Err(e),
            }
        }
        _ => Err(HandlerLoadError::HandlerNotFound),
    }
}
