// this widget is work in progress.
// Items will most likely contain certain messages.

pub struct Item<'a> {
    context: &'a str,
    display_name: &'a str,
}

pub trait ItemsProvider {
    fn context_name(&self) -> &str;
    fn items<'a>(&'a self, query: &str) -> Box<dyn Iterator<Item=Item<'a>>>;
}