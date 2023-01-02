use crate::io::loading_state::LoadingState;
use crate::promise::promise::PromiseState;
use crate::w7e::navcomp_provider::SymbolUsagesPromise;
use crate::widgets::code_results_view::code_results_provider::CodeResultsProvider;

impl CodeResultsProvider for SymbolUsagesPromise {
    fn loading_state(&self) -> LoadingState {
        match self.state() {
            PromiseState::Unresolved => LoadingState::InProgress,
            PromiseState::Ready => LoadingState::Complete,
            PromiseState::Broken => LoadingState::Error,
        }
    }
}