// this widget is work in progress.
// Items will most likely contain certain messages.

use std::rc::Rc;

use crate::widget::any_msg::AnyMsg;

pub trait Item {
    fn display_name(&self) -> Rc<String>;
    fn comment(&self) -> Option<Rc<String>> {
        None
    }
    fn on_hit(&self) -> Box<dyn AnyMsg>;
}

pub trait ItemsProvider {
    fn context_name(&self) -> Rc<String>;

    // TODO(cleanup) Shouldn't query be &str? It's not going to be modified, it doesn't have to be moved
    // either. or maybe the reason is that items is a tailing expression?
    // TODO(cleanup) Do I need a limit if it's iterator?
    fn items(&self, query: String, limit: usize) -> Box<dyn Iterator<Item = Box<dyn Item + '_>> + '_>;
}
