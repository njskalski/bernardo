// this widget is work in progress.
// Items will most likely contain certain messages.

use crate::AnyMsg;
use crate::experiments::beter_deref_str::BetterDerefStr;

pub trait Item {
    fn display_name(&self) -> BetterDerefStr;
    fn comment(&self) -> Option<BetterDerefStr> { None }
    fn on_hit(&self) -> Box<dyn AnyMsg>;
}

pub trait ItemsProvider {
    fn context_name(&self) -> &str;

    // TODO(cleanup) Shouldn't query be &str? It's not going to be modified, it doesn't have to be moved either.
    // or maybe the reason is that items is a tailing expression?
    fn items(&self, query: String, limit: usize) -> Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_>;
}
