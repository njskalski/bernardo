use std::borrow::Cow;
use std::iter::empty;

use log::error;

use crate::w7e::navcomp_provider::{Completion, CompletionsPromise};
use crate::widgets::list_widget::list_widget_item::ListWidgetItem;
use crate::widgets::list_widget::provider::ListItemProvider;

impl ListWidgetItem for Completion {
    fn get_column_name(idx: usize) -> &'static str {
        if idx == 0 {
            "function"
        } else {
            "N/A"
        }
    }

    fn get_min_column_width(idx: usize) -> u16 {
        if idx == 0 { 15 } else {
            error!("requested size of non-existent column");
            0
        }
    }

    fn len_columns() -> usize {
        1
    }

    fn get(&self, idx: usize) -> Option<Cow<'_, str>> {
        if idx == 0 {
            Some(Cow::Borrowed(&self.key))
        } else {
            error!("requested size of non-existent column");
            None
        }
    }
}

impl ListItemProvider<Completion> for CompletionsPromise {
    fn items(&self) -> Box<dyn Iterator<Item=&Completion> + '_> {
        match self.read() {
            None => {
                Box::new(empty())
            }
            Some(vec) => {
                Box::new(vec.into_iter())
            }
        }
    }
}