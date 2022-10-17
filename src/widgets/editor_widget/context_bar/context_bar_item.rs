use std::borrow::Cow;
use std::fmt::{Debug, Formatter};

use crate::widgets::list_widget::list_widget_item::ListWidgetItem;

#[derive(Debug, Clone)]
pub struct ContextBarItem {
    title: String,
}

impl ListWidgetItem for ContextBarItem {
    fn get_column_name(idx: usize) -> &'static str {
        "name"
    }

    fn get_min_column_width(idx: usize) -> u16 {
        match idx {
            0 => 10,
            _ => 0,
        }
    }

    fn len_columns() -> usize {
        1
    }

    fn get(&self, idx: usize) -> Option<Cow<'_, str>> {
        match idx {
            0 => Some(Cow::Borrowed(&self.title)),
            _ => None,
        }
    }
}