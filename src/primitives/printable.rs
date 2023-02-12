use unicode_segmentation::UnicodeSegmentation;

pub trait Printable {
    fn graphemes(&self) -> Box<dyn Iterator<Item=&str> + '_>;
}

impl Printable for str {
    fn graphemes(&self) -> Box<dyn Iterator<Item=&str> + '_> {
        Box::new(UnicodeSegmentation::graphemes(self, true))
    }
}
