use std::borrow::Cow;

use crate::AnyMsg;
use crate::experiments::wrapped_future::WrappedFuture;
use crate::w7e::navcomp_provider::Completion;
use crate::widget::any_msg::AsAny;
use crate::widgets::editor_widget::completion::msg::CompletionWidgetMsg;
use crate::widgets::fuzzy_search::helpers::is_subsequence;
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};

impl Item for Completion {
    fn display_name(&self) -> Cow<str> {
        self.key.into()
    }

    fn comment(&self) -> Option<Cow<str>> {
        self.desc.map(|c| c.into())
    }

    fn on_hit(&self) -> Box<dyn AnyMsg> {
        CompletionWidgetMsg::Selected(self.clone()).boxed()
    }
}

impl ItemsProvider for WrappedFuture<Vec<Completion>> {
    fn context_name(&self) -> &str {
        "lsp"
    }

    fn items(&self, query: String, limit: usize) -> Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_> {
        self.read().map(|vec| {
            Box::new(
                vec
                    .into_iter()
                    .filter(|c| is_subsequence(&c.key, &query))
                    .take(limit)
                    .map(|c| Box::new(c.clone()) as Box<dyn Item>)
            ) as Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_>
        }).unwrap_or(Box::new(std::iter::empty()))
    }
}