use std::string::String;

use crate::io::buffer_output::BufferOutput;
use crate::io::cell::Cell;
use crate::io::style::TextStyle;
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
            let res: (XY, &'a Cell) = (self.pos.clone(), &self.buffer[self.pos].cell);

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

pub struct BufferOutputSubsequenceIter<'a> {
    buffer: &'a BufferOutput,
    text_style: TextStyle,
    pos: XY,
}

impl<'a> BufferOutputSubsequenceIter<'a> {
    pub fn new(buffer: &'a BufferOutput, text_style: TextStyle) -> Self {
        BufferOutputSubsequenceIter {
            buffer,
            text_style,
            pos: XY::ZERO,
        }
    }
}

// TODO test
impl<'a> Iterator for BufferOutputSubsequenceIter<'a> {
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
                    let cell = &self.buffer[self.pos].cell;
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

#[cfg(test)]
mod tests {
    use crate::config::theme::Theme;
    use crate::io::buffer_output::{BufferOutput, CellPair};
    use crate::io::cell::Cell;
    use crate::io::ext_info::ExtInfo;
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
                    buffer[XY::new(x, y)] = CellPair { cell: b.clone(), ext: ExtInfo::default() };
                } else {
                    buffer[XY::new(x, y)] = CellPair { cell: a.clone(), ext: ExtInfo::default() };
                }
            }
        }

        for x in 0..10 as u16 {
            buffer[XY::new(x, 1)] = CellPair { cell: b.clone(), ext: ExtInfo::default() };
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

        buffer[XY::new(0, 0)] = CellPair { cell: b.clone(), ext: ExtInfo::default() };
        buffer[XY::new(0, 1)] = CellPair { cell: a.clone(), ext: ExtInfo::default() };
        buffer[XY::new(1, 0)] = CellPair { cell: a.clone(), ext: ExtInfo::default() };
        buffer[XY::new(1, 1)] = CellPair { cell: b.clone(), ext: ExtInfo::default() };


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