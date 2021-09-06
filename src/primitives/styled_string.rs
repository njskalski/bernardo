use crate::io::style::{TextStyle, TextStyle_WhiteOnBlack};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use std::slice::Iter;
use crate::primitives::xy::XY;
use std::alloc::handle_alloc_error;

pub struct StyledSubString {
    pub text: String,
    pub style : TextStyle,
}

impl StyledSubString {
    pub fn flat(&self) -> bool {
        self.text.contains("\n")
    }
}

pub struct StyledString {
    lines : Vec<Vec<StyledSubString>>,
}

impl StyledString {
    pub fn empty() -> Self {
        StyledString {
            lines : vec![],
        }
    }

    pub fn with(mut self, new_line : bool, text : String, style : TextStyle) -> Self {
        let mut lines = self.lines;
        let new_piece = StyledSubString{ text, style };

        if new_line {
            lines.push(vec![new_piece]);
        } else {
            if lines.is_empty() {
                lines.push(vec![]);
            }

            lines.last_mut().unwrap().push(new_piece);
        }

        StyledString { lines }
    }

    pub fn len(&self) -> usize {
        self.substrings.iter().fold(0, |acc, ss| acc + ss.text.len())
    }

    // This is width assuming there is no newlines.
    pub fn flat_width(&self) -> usize {
        self.substrings.iter().fold(0,
        |acc, ss| acc + ss.text.width())
    }

    pub fn substrings(&self) -> Iter<StyledSubString> {
        self.substrings.iter()
    }

    pub fn is_flat(&self) -> bool {
        self.lines.len() < 2
    }

    pub fn size(&self) -> XY {
        // TODO add cache in cell

        let mut width: usize = 0;
        let mut height: usize = 0;

        let mut curr_line_width : usize = 0;
        for ssi in self.substrings() {
            for piece in ssi.text.split("\n") {
                curr_line_width += piece.width();

                // breaking line
                if curr_line_width > width {
                    width = curr_line_width;
                }
                height += 1;
                curr_line_width = 0;
            }
        }

        if curr_line_width > width {
            width = curr_line_width;
        }

        // TODO add some failsafes so it never panics

        XY::new(width as u16, height as u16)
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::styled_string::StyledString;
    use crate::io::style::TextStyle_WhiteOnBlack;
    use crate::primitives::xy::XY;

    fn simple_styled_string(text : &str) -> StyledString {
        StyledString::empty().with(
            text.to_string(),
            TextStyle_WhiteOnBlack,
        )
    }

    #[test]
    fn styled_string_size_test1() {
        let ss = simple_styled_string("hello world");

        assert_eq!(ss.is_flat(), true);
        assert_eq!(ss.len(), 11);
        assert_eq!(ss.size(), XY::new(11,1));
    }

    #[test]
    fn styled_string_size_test2() {
        let ss = simple_styled_string("hello\nworld");

        assert_eq!(ss.is_flat(), true);
        assert_eq!(ss.len(), 11);
        assert_eq!(ss.size(), XY::new(5,2));
    }

    #[test]
    fn styled_string_size_test3() {
        let ss = simple_styled_string("hel").with("lo\nworld".to_string(), TextStyle_WhiteOnBlack);

        assert_eq!(ss.is_flat(), true);
        assert_eq!(ss.len(), 11);
        assert_eq!(ss.size(), XY::new(5,2));
    }
}