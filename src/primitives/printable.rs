use std::rc::Rc;
use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::experiments::grapheme_lines_streaming_iterator::GraphemeLinesStreamingIterator;

pub trait Printable {
    fn graphemes(&self) -> Box<dyn Iterator<Item = &str> + '_>;

    fn screen_width(&self) -> u16 {
        let mut res = 0 as u16;
        for g in self.graphemes() {
            if res as usize + g.width() > u16::MAX as usize {
                return u16::MAX;
            }

            res += g.width() as u16;
        }

        res
    }

    fn lines(&self) -> GraphemeLinesStreamingIterator {
        GraphemeLinesStreamingIterator::new(self.graphemes())
    }

    fn to_owned_string(&self) -> String {
        let mut s = String::new();

        for g in self.graphemes() {
            s += g;
        }

        s
    }
}

impl Printable for &str {
    fn graphemes(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(UnicodeSegmentation::graphemes(*self, true))
    }
}

impl Printable for Rc<String> {
    fn graphemes(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(UnicodeSegmentation::graphemes(self.as_str(), true))
    }
}

impl Printable for Arc<String> {
    fn graphemes(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(UnicodeSegmentation::graphemes(self.as_str(), true))
    }
}

impl Printable for String {
    fn graphemes(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(UnicodeSegmentation::graphemes(self.as_str(), true))
    }
}
