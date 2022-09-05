use log::debug;
use tokio::sync::mpsc::UnboundedSender;

use crate::ConfigRef;
use crate::fs::path::SPath;
use crate::w7e::handler::Handler;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_group::NavCompTick;
use crate::w7e::rust::handler_rust::RustHandler;

/*
This is a single point of entry to loading LanguageHandlers, to be used by both workspace generator
    and deserializer
 */
pub async fn load_handler(config: &ConfigRef,
                          handler_id: &str,
                          ff: SPath,
                          navcomp_tick_sender: UnboundedSender<NavCompTick>,
) -> Result<Box<dyn Handler>, HandlerLoadError> {
    debug!("attempting to load handler {} for {:?}", handler_id, ff.absolute_path());
    match handler_id {
        "rust" => {
            // RustHandler::load(ff).map(|handler| Ok(Box::new(handler) as Box<dyn Handler>))
            match RustHandler::load(config,
                                    ff,
                                    navcomp_tick_sender,
            ).await {
                Ok(o) => Ok(Box::new(o)),
                Err(e) => Err(e),
            }
        }
        _ => Err(HandlerLoadError::HandlerNotFound),
    }
}
