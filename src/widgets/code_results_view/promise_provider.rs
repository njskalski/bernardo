use std::iter;

use log::debug;

use crate::io::loading_state::LoadingState;
use crate::promise::promise::PromiseState;
use crate::w7e::navcomp_provider::{SymbolUsage, SymbolUsagesPromise};
use crate::widgets::code_results_view::code_results_provider::CodeResultsProvider;

impl CodeResultsProvider for SymbolUsagesPromise {
    fn poll(&mut self) {
        let update_result = self.update();
        debug!("ticking result: {:?}", update_result);
    }

    fn loading_state(&self) -> LoadingState {
        match self.state() {
            PromiseState::Unresolved => LoadingState::InProgress,
            PromiseState::Ready => LoadingState::Complete,
            PromiseState::Broken => LoadingState::Error,
        }
    }

    // TODO this entire method is a stub. It should not copy, it should stream and stuff.
    fn items(&self) -> Box<dyn Iterator<Item=SymbolUsage> + '_> {
        match self.read() {
            None => {
                Box::new(iter::empty()) as Box<dyn Iterator<Item=SymbolUsage>>
            }
            Some(vec) => {
                debug!("returning {} results", vec.len());
                Box::new(vec.iter().map(|c| c.clone()))
            }
        }
    }
}