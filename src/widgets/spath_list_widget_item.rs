use std::rc::Rc;

use log::warn;

use crate::fs::path::SPath;
use crate::widgets::list_widget::list_widget_item::ListWidgetItem;

impl ListWidgetItem for SPath {
    fn get_column_name(idx: usize) -> &'static str {
        match idx {
            0 => "filename",
            1 => "size",
            2 => "type",
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

    fn get(&self, idx: usize) -> Option<Rc<String>> {
        match idx {
            0 => Some(Rc::new(self.label().to_string())), //TODO
            1 => Some(Rc::new("N/A".to_string())),
            2 => Some(Rc::new("N/A".to_string())),
            _ => None,
        }
    }
}
