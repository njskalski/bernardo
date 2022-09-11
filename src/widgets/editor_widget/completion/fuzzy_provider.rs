use crate::experiments::wrapped_future::WrappedFuture;
use crate::w7e::navcomp_provider::Completion;
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};

impl ItemsProvider for WrappedFuture<Vec<Completion>> {
    fn context_name(&self) -> &str {
        todo!()
    }

    fn items(&self, query: String, limit: usize) -> Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_> {
        todo!()
    }
}