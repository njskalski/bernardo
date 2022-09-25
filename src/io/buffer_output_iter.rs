use std::string::String;

use crate::io::buffer_output::BufferOutput;
use crate::io::cell::Cell;
use crate::io::style::TextStyle;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub struct BufferOutputIter<'a> {
    buffer: &'a BufferOutput,
    text_style: TextStyle,
    pos: XY,
}

impl<'a> BufferOutputIter<'a> {
    pub fn new(buffer: &'a BufferOutput, text_style: TextStyle) -> Self {
        BufferOutputIter {
            buffer,
            text_style,
            pos: XY::ZERO,
        }
    }
}

// TODO test
impl<'a> Iterator for BufferOutputIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.buffer.size() {
            None
        } else {
            while self.pos < self.buffer.size() {
                if self.pos.x >= self.buffer.size().x {
                    if self.pos.y < self.buffer.size().y {
                        self.pos.x = 0;
                        self.pos.y += 1;
                    } else {
                        self.pos = self.buffer.size();
                        return None;
                    }
                }

                let mut result = String::new();

                'sticking:
                for x in self.pos.x..self.buffer.size().x {
                    let pos = XY::new(x, self.pos.y);
                    let cell = &self.buffer[pos];
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
    use crate::io::buffer_output::BufferOutput;
    use crate::io::cell::Cell;
    use crate::io::style::TextStyle;
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
            for y in 0..4 as u16 {
                buffer[XY::new(x, y)] = a;
            }
        }
    }
}