use std::iter;

use log::debug;

use crate::io::loading_state::LoadingState;
use crate::primitives::printable::Printable;
use crate::promise::promise::{PromiseState, UpdateResult};
use crate::w7e::navcomp_provider::{SymbolUsage, SymbolUsagesPromise};
use crate::widgets::code_results_view::code_results_provider::{CodeResultsProvider, PollResult};

#[derive(Debug)]
pub struct WrappedSymbolUsagesPromise {
    symbol: String,
    promise: SymbolUsagesPromise,
}

impl WrappedSymbolUsagesPromise {
    pub fn new(symbol: String, promise: SymbolUsagesPromise) -> Self {
        WrappedSymbolUsagesPromise {
            symbol,
            promise,
        }
    }
}

impl CodeResultsProvider for WrappedSymbolUsagesPromise {
    fn description(&self) -> Box<dyn Printable> {
        Box::new(format!("Usages of symbol \"{}\"", &self.symbol))
    }

    fn poll(&mut self) -> PollResult {
        let old_state = self.loading_state();
        let update_result = self.promise.update();
        debug!("ticking result: {:?}", update_result);
        let new_state = self.loading_state();
        PollResult {
            old_state,
            new_state,
        }
    }

    fn loading_state(&self) -> LoadingState {
        match self.promise.state() {
            PromiseState::Unresolved => LoadingState::InProgress,
            PromiseState::Ready => LoadingState::Complete,
            PromiseState::Broken => LoadingState::Error,
        }
    }

    // TODO this entire method is a stub. It should not copy, it should stream and stuff.
    fn items(&self) -> Box<dyn Iterator<Item=SymbolUsage> + '_> {
        match self.promise.read() {
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