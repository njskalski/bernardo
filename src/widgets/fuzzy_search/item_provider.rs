// this widget is work in progress.
// Items will most likely contain certain messages.

use crossterm::style::ContentStyle;
use unicode_segmentation::UnicodeSegmentation;

use crate::Theme;

pub struct Item<'a> {
    context: &'a str,
    display_name: &'a str,
}

impl<'a> Item<'a> {
    pub fn display_name(&self) -> &str {
        self.display_name
    }
}

pub trait ItemsProvider {
    fn context_name(&self) -> &str;
    fn items<'a>(&'a self, query: &str) -> Box<dyn Iterator<Item=Item<'a>>>;
}
