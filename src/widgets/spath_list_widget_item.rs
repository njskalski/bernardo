use log::warn;

use crate::fs::path::SPath;
use crate::widgets::fuzzy_search::item_provider::Item;
use crate::widgets::list_widget::ListWidgetItem;

impl ListWidgetItem for SPath {
    fn get_column_name(idx: usize) -> &'static str {
        match idx {
            0 => { "filename" }
            1 => { "size" }
            2 => { "type" }
            _ => {
                warn!("requested index > 2 for SPath in ListWidgetItem");
                "N/A"
            }
        }
    }

    fn get_min_column_width(_idx: usize) -> u16 {
        10 // TODO completely arbitrary
    }

    fn len_columns() -> usize {
        3
    }

    fn get(&self, idx: usize) -> Option<String> {
        match idx {
            0 => Some(self.display_name().to_string()), // TODO,
            1 => Some("N/A".to_string()),
            2 => Some("N/A".to_string()),
            _ => None
        }
    }
}