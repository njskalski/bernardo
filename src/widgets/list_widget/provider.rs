use std::fmt::Debug;
use std::iter::empty;

use crate::widgets::list_widget::list_widget_item::ListWidgetItem;

pub trait Provider<Item: ListWidgetItem>: Debug {
    fn iter(&self) -> Box<dyn Iterator<Item=&Item> + '_>;
}

impl<Item: ListWidgetItem> Provider<Item> for () {
    fn iter(&self) -> Box<dyn Iterator<Item=&Item> + '_> {
        Box::new(empty())
    }
}
//
// impl<Item: ListWidgetItem> Provider<Item> for Vec<Item> {
//     fn iter(&self) -> Box<dyn Iterator<Item=&Item> + '_> {
//         Box::new(self.iter())
//     }
// }