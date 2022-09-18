use std::fmt::{Debug, Formatter};

use log::error;

use crate::w7e::navcomp_provider::{Completion, CompletionsPromise};
use crate::widgets::list_widget::list_widget_item::ListWidgetItem;
use crate::widgets::list_widget::list_widget_provider::ListWidgetProvider;

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

    fn get(&self, idx: usize) -> Option<String> {
        if idx == 0 {
            Some(self.key.clone())
        } else {
            error!("requested size of non-existent column");
            None
        }
    }
}

impl ListWidgetProvider<Completion> for CompletionsPromise {
    fn len(&self) -> usize {
        match self.read() {
            None => 0,
            Some(res) => res.len(),
        }
    }

    fn get(&self, idx: usize) -> Option<&Completion> {
        self.read().map(|res| res.get(idx)).flatten()
    }
}