use crate::config::config::ConfigRef;
use crate::gladius::navcomp_loader::NavCompLoader;
use crate::w7e::handler::Handler;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_group::NavCompTickSender;
use crate::w7e::project_scope::ProjectScope;

pub struct MockNavcompLoader {}

impl MockNavcompLoader {
    pub fn new() -> Self {
        MockNavcompLoader {}
    }
}

impl NavCompLoader for MockNavcompLoader {
    fn load_handler(&self, config: &ConfigRef, project_scope: &ProjectScope, navcomp_tick_sender: NavCompTickSender) -> Result<Box<dyn Handler>, HandlerLoadError> {
        todo!()
    }
}