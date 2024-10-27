use std::sync::Arc;

use crate::io::loading_state::LoadingState;
use crate::primitives::printable::Printable;
use crate::primitives::symbol_usage::SymbolUsage;
use crate::promise::streaming_promise::{StreamingPromise, StreamingPromiseState};
use crate::widgets::code_results_view::code_results_provider::CodeResultsProvider;

#[derive(Debug)]
pub struct FullTextSearchCodeResultsProvider {
    desc: Arc<String>,
    promise: Box<dyn StreamingPromise<SymbolUsage>>,
}

impl FullTextSearchCodeResultsProvider {
    pub fn new(desc: Arc<String>, promise: Box<dyn StreamingPromise<SymbolUsage>>) -> Self {
        Self { desc, promise }
    }

    pub fn boxed(self) -> Box<dyn CodeResultsProvider> {
        Box::new(self)
    }
}

impl CodeResultsProvider for FullTextSearchCodeResultsProvider {
    fn description(&self) -> Box<dyn Printable> {
        Box::new(self.desc.clone())
    }

    fn poll(&mut self) {
        self.promise.update();
    }

    fn loading_state(&self) -> LoadingState {
        match self.promise.state() {
            StreamingPromiseState::Streaming => LoadingState::InProgress,
            StreamingPromiseState::Finished => LoadingState::Complete,
            StreamingPromiseState::Broken => LoadingState::Error,
        }
    }

    fn items(&self) -> Box<dyn Iterator<Item = &SymbolUsage> + '_> {
        Box::new(self.promise.read().iter())
    }
}
