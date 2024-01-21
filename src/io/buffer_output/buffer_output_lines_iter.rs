use std::string::String;

use crate::io::buffer_output::buffer_output::BufferOutput;
use crate::io::buffer_output::horizontal_iter_item::HorizontalIterItem;
use crate::io::cell::Cell;
use crate::io::style::TextStyle;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub struct BufferLinesIter<'a> {
    buffer: &'a BufferOutput,
    rect: Rect,
    pos: XY,
    // If style is set, only lines conforming to this style in it's entirety will be allowed.
    style_op: Option<TextStyle>,
}

impl<'a> BufferLinesIter<'a> {
    pub fn new(buffer: &'a BufferOutput) -> Self {
        let rect = Rect::new(XY::ZERO, buffer.size());
        BufferLinesIter {
            buffer,
            rect,
            pos: XY::ZERO,
            style_op: None,
        }
    }

    pub fn with_rect(self, rect: Rect) -> Self {
        Self {
            rect,
            pos: rect.pos,
            ..self
        }
    }

    pub fn with_style(self, text_style: TextStyle) -> Self {
        Self {
            style_op: Some(text_style),
            ..self
        }
    }
}

impl<'a> Iterator for BufferLinesIter<'a> {
    type Item = HorizontalIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        'primary: while self.pos.y < self.rect.lower_right().y {
            let mut result = String::new();
            let mut style: Option<TextStyle> = None;
            let mut style_never_set = true;
            let mut begin_pos: Option<XY> = None;

            result.reserve(self.rect.size.x as usize);

            for x in self.rect.pos.x..self.rect.lower_right().x {
                let pos = XY::new(x, self.pos.y);
                let cell = &self.buffer[pos];
                match cell {
                    Cell::Begin {
                        style: cell_style,
                        grapheme,
                    } => {
                        if let Some(set_style) = self.style_op {
                            if set_style != *cell_style {
                                self.pos.y += 1;
                                continue 'primary;
                            }
                        }

                        // from here
                        if begin_pos.is_none() {
                            begin_pos = Some(pos);
                        }
                        if let Some(old_style) = &style {
                            if old_style != cell_style {
                                style = None;
                            }
                        }
                        if style_never_set {
                            style = Some(cell_style.clone());
                            style_never_set = false;
                        }
                        // to here is just setting data

                        result += grapheme;
                    }
                    Cell::Continuation => {}
                }
            }

            self.pos.y += 1;
            return Some(HorizontalIterItem {
                absolute_pos: begin_pos.unwrap(),
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
    use crate::primitives::rect::Rect;
    use crate::primitives::xy::XY;

    #[test]
    fn test_buffer_output_lines_iter_1() {
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
    fn test_buffer_output_lines_iter_2() {
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
    fn test_buffer_output_lines_iter_3() {
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

        let mut iter = buffer
            .lines_iter()
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
