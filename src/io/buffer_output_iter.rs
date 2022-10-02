use std::string::String;

use crate::io::buffer_output::BufferOutput;
use crate::io::cell::Cell;
use crate::io::style::TextStyle;
use crate::primitives::common_query::CommonQuery::String;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub struct BufferOutputCellsIter<'a> {
    buffer: &'a BufferOutput,
    pos: XY,
}

impl<'a> BufferOutputCellsIter<'a> {
    pub fn new(buffer: &'a BufferOutput) -> Self {
        BufferOutputCellsIter {
            buffer,
            pos: XY::ZERO,
        }
    }
}

impl<'a> Iterator for BufferOutputCellsIter<'a> {
    type Item = (XY, &'a Cell);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.buffer.size() {
            None
        } else {
            let res: (XY, &'a Cell) = (self.pos.clone(), &self.buffer[self.pos]);

            self.pos.x += 1;

            if self.pos.x == self.buffer.size().x {
                if self.pos.y + 1 < self.buffer.size().y {
                    self.pos.x = 0;
                    self.pos.y += 1;
                    debug_assert!(self.pos.y <= self.buffer.size().y);
                } else {
                    self.pos = self.buffer.size();
                }
            }

            Some(res)
        }
    }
}

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

// TODO test
impl<'a> Iterator for BufferStyleIter<'a> {
    type Item = String;

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

                'sticking:
                for x in self.pos.x..self.buffer.size().x {
                    let cell = &self.buffer[self.pos];
                    self.pos = XY::new(x + 1, self.pos.y);
                    debug_assert!(self.pos.x <= self.buffer.size().x);


                    if let Cell::Begin { style, grapheme } = cell {
                        if *style == self.text_style {
                            result += grapheme;
                        } else {
                            if !result.is_empty() {
                                break 'sticking;
                            }
                        }
                    }
                }

                if !result.is_empty() {
                    return Some(result);
                }
            }

            None
        }
    }
}

pub struct BufferLinesIter<'a> {
    buffer: &'a BufferOutput,
    rect: Rect,
    pos: XY,
}

impl<'a> BufferLinesIter<'a> {
    pub fn new(buffer: &'a BufferOutput) -> Self {
        let rect = Rect::new(XY::ZERO, buffer.size());
        BufferLinesIter {
            buffer,
            rect,
            pos: XY::ZERO,
        }
    }

    pub fn with_rect(self, rect: Rect) -> Self {
        Self {
            rect,
            ..self
        }
    }
}

impl<'a> Iterator for BufferLinesIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos.y < self.rect.lower_right().y {
            let mut result = String::new();
            result.reserve(self.rect.size.x as usize);

            for x in self.rect.size.x..self.rect.lower_right().x {
                let pos = XY::new(x, self.pos.y);
                let cell = &self.buffer[pos];
                match cell {
                    Cell::Begin { style, grapheme } => {
                        result += grapheme;
                    }
                    Cell::Continuation => {}
                }
            }

            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::theme::Theme;
    use crate::io::buffer_output::BufferOutput;
    use crate::io::cell::Cell;
    use crate::primitives::xy::XY;

    #[test]
    fn test_buffer_output_iter() {
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
        assert_eq!(iter.next(), Some("bbb".to_string()));
        assert_eq!(iter.next(), Some("bb".to_string()));

        assert_eq!(iter.next(), Some("bbbbbbbbbb".to_string()));

        assert_eq!(iter.next(), Some("bbb".to_string()));
        assert_eq!(iter.next(), Some("bb".to_string()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_buffer_output_cell_iter() {
        let theme = Theme::default();
        let non_focused = theme.ui.non_focused;
        let focused = theme.ui.focused;

        let mut buffer: BufferOutput = BufferOutput::new(XY::new(2, 2));

        let a = Cell::new(non_focused, "a".to_string());
        let b = Cell::new(focused, "b".to_string());

        buffer[XY::new(0, 0)].set(&b);
        buffer[XY::new(0, 1)].set(&a);
        buffer[XY::new(1, 0)].set(&a);
        buffer[XY::new(1, 1)].set(&b);


        /*
         01234567890
        0ba
        1ab
        2
        3
         */

        let mut iter = buffer.cells_iter();
        assert_eq!(iter.next(), Some((XY::new(0, 0), &b)));
        assert_eq!(iter.next(), Some((XY::new(1, 0), &a)));
        assert_eq!(iter.next(), Some((XY::new(0, 1), &a)));
        assert_eq!(iter.next(), Some((XY::new(1, 1), &b)));
        assert_eq!(iter.next(), None);
    }
}