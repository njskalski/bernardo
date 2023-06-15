use std::string::String;

use crate::io::buffer_output::buffer_output::BufferOutput;
use crate::io::cell::Cell;
use crate::io::style::TextStyle;
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

#[cfg(test)]
mod tests {
    use crate::config::theme::Theme;
    use crate::io::buffer_output::buffer_output::BufferOutput;
    use crate::io::cell::Cell;
    use crate::primitives::rect::Rect;
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
        assert_eq!(iter.next().unwrap().text, "bbb");
        assert_eq!(iter.next().unwrap().text, "bb");

        assert_eq!(iter.next().unwrap().text, "bbbbbbbbbb");

        assert_eq!(iter.next().unwrap().text, "bbb");
        assert_eq!(iter.next().unwrap().text, "bb");
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

    #[test]
    fn test_buffer_lines_iter_1() {
        let theme = Theme::default();
        let non_focused = theme.ui.non_focused;
        let focused = theme.ui.focused;

        let mut buffer: BufferOutput = BufferOutput::new(XY::new(10, 3));

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

        let mut iter = buffer.lines_iter();
        let first_item = iter.next().unwrap();
        assert_eq!(first_item.text, "bbbaaaaabb");
        assert_eq!(first_item.absolute_pos, XY::ZERO);
        assert_eq!(first_item.text_style, None);

        let second_item = iter.next().unwrap();
        assert_eq!(second_item.text, "bbbbbbbbbb");
        assert_eq!(second_item.absolute_pos, XY::new(0, 1));
        assert_eq!(second_item.text_style.as_ref(), b.style());

        let third_item = iter.next().unwrap();
        assert_eq!(third_item.text, "bbbaaaaabb");
        assert_eq!(third_item.absolute_pos, XY::new(0, 2));
        assert_eq!(third_item.text_style, None);

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_buffer_lines_iter_2() {
        let theme = Theme::default();
        let non_focused = theme.ui.non_focused;
        let focused = theme.ui.focused;

        let mut buffer: BufferOutput = BufferOutput::new(XY::new(10, 3));

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

        let mut iter = buffer.lines_iter().with_rect(Rect::new(XY::new(1, 1), XY::new(4, 2)));

        let first_item = iter.next().unwrap();
        assert_eq!(first_item.text, "bbbb");
        assert_eq!(first_item.absolute_pos, XY::new(1, 1));
        assert_eq!(first_item.text_style.as_ref(), b.style());

        let second_item = iter.next().unwrap();
        assert_eq!(second_item.text, "bbaa");
        assert_eq!(second_item.absolute_pos, XY::new(1, 2));
        assert_eq!(second_item.text_style, None);

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_buffer_lines_iter_3() {
        let theme = Theme::default();
        let non_focused = theme.ui.non_focused;
        let focused = theme.ui.focused;

        let mut buffer: BufferOutput = BufferOutput::new(XY::new(10, 3));

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

        let mut iter = buffer.lines_iter()
            .with_rect(Rect::new(XY::new(3, 0), XY::new(4, 3)))
            .with_style(*a.style().unwrap());

        let first_item = iter.next().unwrap();
        assert_eq!(first_item.text, "aaaa");
        assert_eq!(first_item.absolute_pos, XY::new(3, 0));
        assert_eq!(first_item.text_style.as_ref(), a.style());

        let second_item = iter.next().unwrap();
        assert_eq!(second_item.text, "aaaa");
        assert_eq!(second_item.absolute_pos, XY::new(3, 2));
        assert_eq!(first_item.text_style.as_ref(), a.style());

        assert_eq!(iter.next(), None);
    }
}