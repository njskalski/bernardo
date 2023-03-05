use crate::io::loading_state::LoadingState;
use crate::primitives::printable::Printable;
use crate::w7e::navcomp_provider::SymbolUsage;

pub struct PollResult {
    pub old_state: LoadingState,
    pub new_state: LoadingState,
}

impl PollResult {
    pub fn has_first_result(&self) -> bool {
        self.old_state == LoadingState::NotStarted &&
            (self.new_state == LoadingState::InProgress || self.new_state == LoadingState::Complete)
    }
}

/*
Invariant: if items becomes longer, the initial items DO NOT CHANGE.
 */
pub trait CodeResultsProvider {
    fn description(&self) -> Box<dyn Printable>;

    fn poll(&mut self) -> PollResult;

    fn loading_state(&self) -> LoadingState;

    fn items(&self) -> Box<dyn Iterator<Item=SymbolUsage> + '_>;
}