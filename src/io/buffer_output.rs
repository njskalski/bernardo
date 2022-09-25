use std::default::Default;
use std::fmt::{Debug, Formatter};

use log::{debug, warn};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::buffer::Buffer;
use crate::io::buffer_output_iter::BufferOutputIter;
use crate::io::cell::Cell;
use crate::io::output::Output;
use crate::io::style::TextStyle;
use crate::primitives::size_constraint::SizeConstraint;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub type BufferOutput = Buffer<Cell>;

impl Output for BufferOutput {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        if !self.size_constraint().strictly_bigger_than(pos) {
            warn!(
                "early exit on drawing beyond border (req {}, border {:?})",
                pos,
                self.size_constraint()
            );
            return;
        }

        debug_assert!(text.len() < (u16::max_value() as usize));

        let mut offset: u16 = 0;

        for (idx, grapheme) in text.graphemes(true).enumerate() {
            let shift_x = (idx as u16) + offset;

            match self.size_constraint().x() {
                Some(max_x) => {
                    if pos.x + shift_x >= max_x {
                        break;
                    }

                    if (max_x as usize) - ((pos.x + shift_x) as usize) < grapheme.width() {
                        debug!("early quit on wide char.");
                        break;
                    }
                }
                None => {}
            }


            let xy = pos + XY::new(shift_x as u16, 0);
            self[xy] = Cell::Begin {
                style,
                grapheme: grapheme.to_string(),
            };

            // TODO this can go outside the line or even cause a panic.
            if grapheme.width() > 0 {
                for _ in 0..grapheme.width() - 1 {
                    offset += 1;
                    let cont_shift_x = (idx as u16) + offset;
                    let xy2 = pos + XY::new(cont_shift_x as u16, 0 as u16);

                    self[xy2] = Cell::continuation();
                }
            }
        }
    }

    fn clear(&mut self) -> Result<(), std::io::Error> {
        for idx in 0..self.cells().len() {
            self.cells_mut()[idx] = Cell::default();
        }
        Ok(())
    }

    fn size_constraint(&self) -> SizeConstraint {
        SizeConstraint::simple(self.size())
    }
}

impl Debug for BufferOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[BufferOutput {}]", self.size())
    }
}

impl BufferOutput {
    pub fn items_of_style(&self, style: TextStyle) -> BufferOutputIter {
        BufferOutputIter::new(&self, style)
    }
}

impl ToString for BufferOutput {
    fn to_string(&self) -> String {
        let mut wchujdlugistring = String::new();
        wchujdlugistring.reserve((self.size().x * self.size().y + 1) as usize);

        for x in 0..self.size().x {
            for y in 0..self.size().y {
                let pos = XY::new(x, y);
                let cell = &self[pos];
                match cell {
                    Cell::Begin { style, grapheme } => {
                        wchujdlugistring += grapheme;
                    }
                    Cell::Continuation => {}
                }
            }
            wchujdlugistring += "\n";
        }

        wchujdlugistring
    }
}

