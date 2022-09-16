use std::borrow::Cow;
use std::sync::{Arc, RwLock};

use log::error;

use crate::AnyMsg;
use crate::experiments::wrapped_future::WrappedFuture;
use crate::w7e::navcomp_provider::Completion;
use crate::widget::any_msg::AsAny;
use crate::widgets::editor_widget::completion::msg::CompletionWidgetMsg;
use crate::widgets::fuzzy_search::helpers::is_subsequence;
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};

// TODO tu jest tak nasrane, ze szkoda gadac. Problem polega na tym, ze fuzzy wspoldzieli
//  providera z samym widgetem, wiec schowalem to za ArcRwLock co strasznie zaciemnia kod.

impl Item for Completion {
    fn display_name(&self) -> Cow<str> {
        self.key.clone().into()
    }

    fn comment(&self) -> Option<Cow<str>> {
        self.desc.as_ref().map(|c| c.into())
    }

    fn on_hit(&self) -> Box<dyn AnyMsg> {
        CompletionWidgetMsg::Selected(self.clone()).boxed()
    }
}

impl ItemsProvider for Arc<RwLock<WrappedFuture<Vec<Completion>>>> {
    fn context_name(&self) -> &str {
        "lsp"
    }

    fn items(&self, query: String, limit: usize) -> Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_> {
        // bosh jak tu jest nasrane

        let rlock = match self.try_read() {
            Ok(r) => r,
            Err(_) => {
                error!("failed acquiring read lock");
                return Box::new(std::iter::empty());
            }
        };

        rlock.read().map(move |vec| {
            Box::new(
                vec
                    .iter()
                    .filter(move |c| is_subsequence(&c.key, &query))
                    .take(limit)
                    .map(|c| Box::new(c.clone()) as Box<dyn Item>)
                    .collect::<Vec<_>>()
                    .into_iter()
            ) as Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_>
        }).unwrap_or(Box::new(std::iter::empty()))
    }
}