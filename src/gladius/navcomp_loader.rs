use crate::config::config::ConfigRef;
use crate::experiments::buffer_register::BufferRegisterRef;
use crate::w7e::handler::Handler;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_group::NavCompTickSender;
use crate::w7e::project_scope::ProjectScope;

pub trait NavCompLoader: Send + Sync + 'static {
    fn load_handler(
        &self,
        config: &ConfigRef,
        buffer_register_ref: &BufferRegisterRef,
        project_scope: &ProjectScope,
        navcomp_tick_sender: NavCompTickSender,
    ) -> Result<Box<dyn Handler>, HandlerLoadError>;
}
