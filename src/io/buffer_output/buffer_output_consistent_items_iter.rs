use std::string::String;

use crate::io::buffer_output::buffer_output::BufferOutput;
use crate::io::buffer_output::horizontal_iter_item::ConsistentHorizontalItem;
use crate::io::cell::Cell;
use crate::io::style::TextStyle;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

// This iterates over items with consistent text style, ie "breaks always when style changes"

// invariants:
// - never points on cell type "continuation"
pub struct BufferConsistentItemsIter<'a> {
    buffer: &'a BufferOutput,
    rect: Rect,
    pos: XY,
    // style is determined by the position
}

impl<'a> BufferConsistentItemsIter<'a> {
    pub fn new(buffer: &'a BufferOutput) -> Self {
        let rect = Rect::new(XY::ZERO, buffer.size());
        BufferConsistentItemsIter {
            buffer,
            rect,
            pos: XY::ZERO,
        }
    }

    pub fn with_rect(self, rect: Rect) -> Self {
        debug_assert!(self.rect.contains_rect(rect));

        Self {
            rect,
            pos: rect.pos,
            ..self
        }
    }

    pub fn get_style(&self) -> &TextStyle {
        // no continuation => unwrap should be safe
        // this is test code, doesn't have to be user-facing quality
        self.buffer[self.pos].style().unwrap()
    }
}

impl<'a> Iterator for BufferConsistentItemsIter<'a> {
    type Item = ConsistentHorizontalItem;

    fn next(&mut self) -> Option<Self::Item> {
        'y: while self.pos.y < self.rect.lower_right().y {
            let mut result = String::new();
            let style: TextStyle = self.get_style().clone();
            let begin_pos: XY = self.pos;

            'x: for x in begin_pos.x..self.rect.lower_right().x {
                let pos = XY::new(x, self.pos.y);
                let cell = &self.buffer[pos];
                match cell {
                    Cell::Begin {
                        style: cell_style,
                        grapheme,
                    } => {
                        if style == *cell_style {
                            result += grapheme;
                        } else {
                            self.pos = pos;
                            if self.pos.y == self.rect.max_y() {
                                self.pos.y += 1;
                                self.pos.x = self.rect.min_x();
                            }

                            return Some(ConsistentHorizontalItem {
                                absolute_pos: begin_pos,
                                text_style: style,
                                text: result,
                            });
                        }
                    }
                    Cell::Continuation => {}
                }
            }

            self.pos.y += 1;
            self.pos.x = self.rect.min_x();
            return Some(ConsistentHorizontalItem {
                absolute_pos: begin_pos,
                text_style: style,
                text: result,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::config::theme::Theme;
    use crate::io::buffer_output::buffer_output::BufferOutput;
    use crate::io::cell::Cell;
    use crate::primitives::xy::XY;

    #[test]
    fn test_buffer_consistent_items_iter() {
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

        let mut iter = buffer.consistent_items_iter();
        let first_item = iter.next().unwrap();
        assert_eq!(first_item.text, "bbb");
        assert_eq!(first_item.absolute_pos, XY::ZERO);
        assert_eq!(first_item.text_style, focused);

        let second_item = iter.next().unwrap();
        assert_eq!(second_item.text, "aaaaa");
        assert_eq!(second_item.absolute_pos, XY::new(3, 0));
        assert_eq!(second_item.text_style, non_focused);

        let third_item = iter.next().unwrap();
        assert_eq!(third_item.text, "bb");
        assert_eq!(third_item.absolute_pos, XY::new(8, 0));
        assert_eq!(third_item.text_style, focused);

        let fourth_item = iter.next().unwrap();
        assert_eq!(fourth_item.text, "bbbbbbbbbb");
        assert_eq!(fourth_item.absolute_pos, XY::new(0, 1));
        assert_eq!(fourth_item.text_style, focused);

        let fifth_item = iter.next().unwrap();
        assert_eq!(fifth_item.text, "bbb");
        assert_eq!(fifth_item.absolute_pos, XY::new(0, 2));
        assert_eq!(fifth_item.text_style, focused);

        let sixth_item = iter.next().unwrap();
        assert_eq!(sixth_item.text, "aaaaa");
        assert_eq!(sixth_item.absolute_pos, XY::new(3, 2));
        assert_eq!(sixth_item.text_style, non_focused);

        let seventh_item = iter.next().unwrap();
        assert_eq!(seventh_item.text, "bb");
        assert_eq!(seventh_item.absolute_pos, XY::new(8, 2));
        assert_eq!(seventh_item.text_style, focused);

        assert_eq!(iter.next(), None);
    }
}
