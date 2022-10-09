use std::string::String;

use crate::io::buffer_output::BufferOutput;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerticalIterItem {
    pub absolute_pos: XY,
    // Set iff style was consistent over entire item
    pub text_style: Option<TextStyle>,
    pub text: String,
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

impl<'a> Iterator for BufferStyleIter<'a> {
    type Item = VerticalIterItem;

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
                    return Some(VerticalIterItem {
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
    type Item = VerticalIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        'primary:
        while self.pos.y < self.rect.lower_right().y {
            let mut result = String::new();
            let mut style: Option<TextStyle> = None;
            let mut style_never_set = true;
            let mut begin_pos: Option<XY> = None;

            result.reserve(self.rect.size.x as usize);

            for x in self.rect.pos.x..self.rect.lower_right().x {
                let pos = XY::new(x, self.pos.y);
                let cell = &self.buffer[pos];
                match cell {
                    Cell::Begin { style: cell_style, grapheme } => {
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
            return Some(VerticalIterItem {
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
    use crate::io::buffer_output::BufferOutput;
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