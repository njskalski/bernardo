use streaming_iterator::StreamingIterator;

use crate::Widget;
use crate::widget::action_trigger::ActionTrigger;
use crate::widgets::fuzzy_search::item_provider::{FuzzyItem, FuzzyItemsProvider};

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

impl<W: Widget> FuzzyItemsProvider for Actions<W> {
    fn context_name(&self) -> &str {
        todo!()
    }

    fn items(&self, query: String) -> Box<dyn StreamingIterator<Item=Box<dyn FuzzyItem + '_>>> {
        todo!()
    }
}