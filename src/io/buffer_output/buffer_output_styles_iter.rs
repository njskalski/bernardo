use std::string::String;

use crate::io::buffer_output::buffer_output::BufferOutput;
use crate::io::buffer_output::horizontal_iter_item::HorizontalIterItem;
use crate::io::cell::Cell;
use crate::io::style::TextStyle;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

/*
This is an iterator, that uses text_style to FILTER the output. It will skip over items not matching
 */
pub struct BufferStyleIter<'a> {
    buffer: &'a BufferOutput,
    text_style: TextStyle,
    pos: XY,
}

impl<'a> BufferStyleIter<'a> {
    pub fn new(buffer: &'a BufferOutput, text_style: TextStyle) -> Self {
        BufferStyleIter {
            buffer,
            text_style,
            pos: XY::ZERO,
        }
    }
}

impl<'a> Iterator for BufferStyleIter<'a> {
    type Item = HorizontalIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.buffer.size() {
            None
        } else {
            while self.pos.y < self.buffer.size().y {
                if self.pos.x == self.buffer.size().x {
                    if self.pos.y + 1 < self.buffer.size().y {
                        self.pos.x = 0;
                        self.pos.y += 1;
                        debug_assert!(self.pos.y <= self.buffer.size().y);
                    } else {
                        self.pos = self.buffer.size();
                        return None;
                    }
                }

                let mut result = String::new();
                let mut begin: Option<XY> = None;

                'sticking:
                for x in self.pos.x..self.buffer.size().x {
                    let cell = &self.buffer[self.pos];
                    self.pos = XY::new(x + 1, self.pos.y);
                    debug_assert!(self.pos.x <= self.buffer.size().x);

                    if let Cell::Begin { style, grapheme } = cell {
                        if *style == self.text_style {
                            if begin.is_none() {
                                begin = Some(self.pos);
                            }

                            result += grapheme;
                        } else {
                            if !result.is_empty() {
                                break 'sticking;
                            }
                        }
                    }
                }

                if !result.is_empty() {
                    return Some(HorizontalIterItem {
                        absolute_pos: begin.unwrap(),
                        text_style: Some(self.text_style),
                        text: result,
                    });
                }
            }

            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::theme::Theme;
    use crate::io::buffer_output::buffer_output::BufferOutput;
    use crate::io::cell::Cell;
    use crate::primitives::rect::Rect;
    use crate::primitives::xy::XY;

    #[test]
    fn test_buffer_output_styles_iter() {
        let theme = Theme::default();
        let non_focused = theme.ui.non_focused;
        let focused = theme.ui.focused;

        let mut buffer: BufferOutput = BufferOutput::new(XY::new(10, 4));

        let a = Cell::new(non_focused, "a".to_string());
        let b = Cell::new(focused, "b".to_string());

        for x in 0..10 as u16 {
            for y in 0..3 as u16 {
                if x < 3 || x >= 8 {
                    buffer[XY::new(x, y)].set(&b);
                } else {
                    buffer[XY::new(x, y)].set(&a);
                }
            }
        }

        for x in 0..10 as u16 {
            buffer[XY::new(x, 1)].set(&b);
        }

        /*
         01234567890
        0bbbaaaaabb
        1bbbbbbbbbb
        2bbbaaaaabb
        3
         */

        let mut iter = buffer.items_of_style(focused);
        assert_eq!(iter.next().unwrap().text, "bbb");
        assert_eq!(iter.next().unwrap().text, "bb");

        assert_eq!(iter.next().unwrap().text, "bbbbbbbbbb");

        assert_eq!(iter.next().unwrap().text, "bbb");
        assert_eq!(iter.next().unwrap().text, "bb");
        assert_eq!(iter.next(), None);

        // let's test the style is set all over the board if we use style for filtering
        let all_styles_set = buffer.items_of_style(focused).fold(true, |prev, this| {
            prev && this.text_style == Some(focused)
        });

        assert!(all_styles_set);
    }
}