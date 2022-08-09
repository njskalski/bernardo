use crate::Widget;
use crate::widget::action_trigger::ActionTrigger;
use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};

pub struct Actions<W: Widget> {
    vec: Vec<ActionTrigger<W>>,
}

impl<W: Widget> Actions<W> {
    pub fn new(vec: Vec<ActionTrigger<W>>) -> Self {
        Actions {
            vec
        }
    }
}

impl<W: Widget> ItemsProvider for Actions<W> {
    fn context_name(&self) -> &str {
        todo!()
    }

    fn items(&self, query: String, limit: usize) -> Box<dyn Iterator<Item=Box<dyn Item + '_>> + '_> {
        todo!()
    }
}