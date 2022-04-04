// this widget is work in progress.
// Items will most likely contain certain messages.

use crate::AnyMsg;

pub trait Item {
    fn display_name(&self) -> &str;
    fn on_hit(&self) -> Box<dyn AnyMsg>;
}

pub trait ItemsProvider {
    fn context_name(&self) -> &str;
    fn items<'a>(&'a self, query: String) -> Box<dyn Iterator<Item=&'a (dyn Item + 'a)> + '_>;
}
