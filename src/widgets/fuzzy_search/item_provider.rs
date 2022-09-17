// This widget is work in progress.
// Items will most likely contain certain messages.

// There is a reason why Items are NOT unified, we want "everything bar" to be a sum of other bars.
// This can be achieved other way, and just might, but I am not sure yet.

use std::borrow::Cow;

use streaming_iterator::StreamingIterator;

use crate::AnyMsg;

pub trait FuzzyItem {
    fn display_name(&self) -> Cow<str>;
    fn comment(&self) -> Option<Cow<str>> { None }
    fn on_hit(&self) -> Box<dyn AnyMsg>;
}

pub trait FuzzyItemsProvider {
    fn context_name(&self) -> &str;

    // Shouldn't query be &str? It's not going to be modified, it doesn't have to be moved either.
    // ^ no, not really, because in most cases it will be copied from another field (EditBox), with
    // and I don't want to hear "borrowed temporary data"

    fn items(&self, query: String) -> Box<dyn StreamingIterator<Item=Box<dyn FuzzyItem + '_>> + '_>;
}
