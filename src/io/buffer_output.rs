use core::fmt::Alignment::Center;
use std::default::Default;
use std::io::ErrorKind;
use std::ops::{Index, IndexMut};

use log::debug;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::buffer::Buffer;
use crate::io::cell::Cell;
use crate::io::output::Output;
use crate::io::style::TextStyle;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub type BufferOutput = Buffer<Cell>;

impl Output for BufferOutput {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        if pos.x >= self.size().x || pos.y >= self.size().y {
            //TODO
            // debug!(
            //     "early exit on drawing beyond border (req {}, border {})",
            //     pos,
            //     self.size()
            // );
        }

        debug_assert!(pos.x < self.size().x, "pos {}, size {}", pos, self.size());
        debug_assert!(pos.y < self.size().y, "pos {}, size {}", pos, self.size());
        debug_assert!(text.len() < (u16::max_value() as usize));

        let mut offset: u16 = 0;

        for (idx, grapheme) in text.graphemes(true).enumerate() {
            let shift_x = (idx as u16) + offset;

            if pos.x + shift_x >= self.size().x {
                break;
            }

            if (self.size().x as usize) - ((pos.x + shift_x) as usize) < grapheme.width() {
                dbg!("early quit on wide char.");
                break;
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

    fn clear(&mut self) {
        for idx in 0..self.cells().len() {
            self.cells_mut()[idx] = Cell::default();
        }
    }
}
