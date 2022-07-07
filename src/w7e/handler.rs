/*
Handler is a wrapper that translates language specific project definition into common project
items, like run configurations, test targets, and LSP clients.
 */
use std::sync::Arc;
use crate::fs::file_front::FileFront;
use crate::LangId;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_provider::NavCompProvider;

pub type NavCompRef = Arc<Box<dyn NavCompProvider>>;

pub trait Handler {
    fn lang_id(&self) -> LangId;
    fn handler_id(&self) -> &'static str;
    fn project_name(&self) -> &str;

    fn navcomp(&self) -> Option<NavCompRef> { None }
}

