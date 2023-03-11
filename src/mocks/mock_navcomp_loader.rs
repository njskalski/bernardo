use std::sync::{Arc, RwLock};

use crossbeam_channel::Sender;

use crate::config::config::ConfigRef;
use crate::gladius::navcomp_loader::NavCompLoader;
use crate::mocks::mock_navcomp_provider::{MockCompletionMatcher, MockNavCompEvent, MockSymbolMatcher};
use crate::w7e::handler::Handler;
use crate::w7e::handler_load_error::HandlerLoadError;
use crate::w7e::navcomp_group::NavCompTickSender;
use crate::w7e::navcomp_provider::NavCompProvider;
use crate::w7e::project_scope::ProjectScope;
use crate::w7e::rust::handler_rust::RustHandler;

pub struct MockNavcompLoader {
    event_sender: Sender<MockNavCompEvent>,
    completions: Arc<RwLock<Vec<MockCompletionMatcher>>>,
    symbols: Arc<RwLock<Vec<MockSymbolMatcher>>>,
}

impl MockNavcompLoader {
    pub fn new(
        event_sender: Sender<MockNavCompEvent>,
        completions: Arc<RwLock<Vec<MockCompletionMatcher>>>,
        symbols: Arc<RwLock<Vec<MockSymbolMatcher>>>,
    ) -> Self {
        MockNavcompLoader {
            event_sender,
            completions,
            symbols,
        }
    }
}

impl NavCompLoader for MockNavcompLoader {
    fn load_handler(&self, config: &ConfigRef, project_scope: &ProjectScope, navcomp_tick_sender: NavCompTickSender) -> Result<Box<dyn Handler>, HandlerLoadError> {
        debug_assert!(project_scope.handler_id.as_ref() == Some(&"rust".to_string())); // yeah I know it's shit, I have 100 compile errors

        let navcomp_op = Some(
            Arc::new(
                Box::new(
                    crate::mocks::mock_navcomp_provider::MockNavCompProvider::new(
                        navcomp_tick_sender,
                        self.event_sender.clone(),
                        self.completions.clone(),
                    )
                ) as Box<dyn NavCompProvider>)
        );


        Ok(Box::new(
            RustHandler::load(
                config,
                project_scope.path.clone(),
                navcomp_op,
            )?
        ))
    }
}