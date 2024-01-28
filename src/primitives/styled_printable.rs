use unicode_width::UnicodeWidthStr;

use crate::io::style::TextStyle;
use crate::primitives::printable::Printable;

pub trait StyledPrintable {
    fn styled_graphemes(&self) -> Box<dyn Iterator<Item = (&TextStyle, &str)> + '_>;

    fn screen_width(&self) -> u16 {
        let mut res = 0 as u16;
        for (_, g) in self.styled_graphemes() {
            if res as usize + g.width() > u16::MAX as usize {
                return u16::MAX;
            }

            res += g.width() as u16;
        }

        res
    }
}

pub struct StyleWrappedPrintable<P: Printable> {
    style: TextStyle,
    printable: P,
}

impl<P: Printable> StyleWrappedPrintable<P> {
    pub fn new(text_style: TextStyle, printable: P) -> Self {
        StyleWrappedPrintable {
            style: text_style,
            printable,
        }
    }
}

impl<P: Printable> StyledPrintable for StyleWrappedPrintable<P> {
    fn styled_graphemes(&self) -> Box<dyn Iterator<Item = (&TextStyle, &str)> + '_> {
        Box::new(self.printable.graphemes().map(|grapheme| (&self.style, grapheme)))
    }
}

pub struct StyleBorrowedPrintable<'a> {
    style: TextStyle,
    printable: &'a dyn Printable,
}

impl<'a> StyleBorrowedPrintable<'a> {
    pub fn new(text_style: TextStyle, printable: &'a dyn Printable) -> Self {
        StyleBorrowedPrintable {
            style: text_style,
            printable,
        }
    }
}

impl<'a> StyledPrintable for StyleBorrowedPrintable<'a> {
    fn styled_graphemes(&self) -> Box<dyn Iterator<Item = (&TextStyle, &str)> + '_> {
        Box::new(self.printable.graphemes().map(|grapheme| (&self.style, grapheme)))
    }
}

impl StyledPrintable for (TextStyle, String) {
    fn styled_graphemes(&self) -> Box<dyn Iterator<Item = (&TextStyle, &str)> + '_> {
        Box::new(self.1.graphemes().map(|grapeheme| (&self.0, grapeheme)))
    }
}

impl StyledPrintable for Vec<(TextStyle, String)> {
    fn styled_graphemes(&self) -> Box<dyn Iterator<Item = (&TextStyle, &str)> + '_> {
        let mut result: Vec<(&TextStyle, &str)> = vec![];

        for (style, text) in self {
            for grapheme in text.graphemes() {
                result.push((style, grapheme));
            }
        }

        Box::new(result.into_iter())
    }
}

// impl Printable for &str {
//     fn graphemes(&self) -> Box<dyn Iterator<Item=&str> + '_> {
//         Box::new(UnicodeSegmentation::graphemes(*self, true))
//     }
// }
//
// impl Printable for Rc<String> {
//     fn graphemes(&self) -> Box<dyn Iterator<Item=&str> + '_> {
//         Box::new(UnicodeSegmentation::graphemes(self.as_str(), true))
//     }
// }
//
// impl Printable for Arc<String> {
//     fn graphemes(&self) -> Box<dyn Iterator<Item=&str> + '_> {
//         Box::new(UnicodeSegmentation::graphemes(self.as_str(), true))
//     }
// }
//
// impl Printable for String {
//     fn graphemes(&self) -> Box<dyn Iterator<Item=&str> + '_> {
//         Box::new(UnicodeSegmentation::graphemes(self.as_str(), true))
//     }
// }
