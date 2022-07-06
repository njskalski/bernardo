/*
Handler is a wrapper that translates language specific project definition into common project
items, like run configurations, test targets, and LSP clients.
 */
use crate::fs::file_front::FileFront;
use crate::LangId;
use crate::w7e::handler_load_error::HandlerLoadError;

pub trait Handler {
    fn lang_id(&self) -> LangId;
    fn handler_id(&self) -> &'static str;

    fn project_name(&self) -> &str;
}

