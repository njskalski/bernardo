// this widget is work in progress.
// Items will most likely contain certain messages.

use crossterm::style::ContentStyle;
use unicode_segmentation::UnicodeSegmentation;

use crate::Theme;

pub trait Item {
    fn display_name(&self) -> &str;
}

pub trait ItemsProvider {
    fn context_name(&self) -> &str;
    fn items<'a>(&'a self, query: String) -> Box<dyn Iterator<Item=&'a dyn Item> + '_>;
}
