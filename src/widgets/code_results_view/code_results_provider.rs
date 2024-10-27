use crate::io::loading_state::LoadingState;
use crate::primitives::printable::Printable;
use crate::primitives::symbol_usage::SymbolUsage;

/*
Invariant: if items becomes longer, the initial items DO NOT CHANGE.
 */
pub trait CodeResultsProvider {
    fn description(&self) -> Box<dyn Printable>;

    fn poll(&mut self);

    fn loading_state(&self) -> LoadingState;

    fn items(&self) -> Box<dyn Iterator<Item = &SymbolUsage> + '_>;
}
