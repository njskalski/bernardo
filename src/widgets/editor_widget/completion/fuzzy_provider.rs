use std::borrow::Cow;
use std::sync::{Arc, RwLock};

use log::error;

use crate::AnyMsg;
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
