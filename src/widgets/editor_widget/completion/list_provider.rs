use std::fmt::{Debug, Formatter};

use log::error;

use crate::w7e::navcomp_provider::Completion;
use crate::widgets::editor_widget::completion::completion_widget::CompletionsPromise;
use crate::widgets::list_widget::list_widget_item::ListWidgetItem;

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
