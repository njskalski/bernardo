use core::fmt::Alignment::Center;
use std::ops::{Index, IndexMut};

use log::debug;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::cell::Cell;
use crate::io::output::Output;
use crate::io::style::TextStyle;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub struct BufferOutput {
    size: XY,
    cells: Vec<Cell>,
}

impl BufferOutput {
    pub fn new(size: XY) -> Self {
        let mut cells = vec![Cell::empty(); (size.x * size.y) as usize];

        BufferOutput { size, cells }
    }

    fn flatten_index(&self, index: XY) -> usize {
        assert!(index.x < self.size.x);
        assert!(index.y < self.size.y);

        (index.y * self.size.x + index.x) as usize
    }

    fn unflatten_index(&self, index: usize) -> XY {
        assert!(index < u16::max_value() as usize);
        assert!(index < self.cells.len());

        XY::new(index as u16 / self.size.x, index as u16 % self.size.x)
    }
}

impl SizedXY for BufferOutput {
    fn size(&self) -> XY {
        self.size
    }
}

impl Index<XY> for BufferOutput {
    type Output = Cell;

    fn index(&self, index: XY) -> &Self::Output {
        let idx = self.flatten_index(index);
        &self.cells[idx]
    }
}

impl IndexMut<XY> for BufferOutput {
    fn index_mut(&mut self, index: XY) -> &mut Cell {
        let idx = self.flatten_index(index);
        &mut self.cells[idx]
    }
}

impl Output for BufferOutput {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        if pos.x >= self.size.x || pos.y >= self.size.y {
            //TODO
            debug!(
                "early exit on drawing beyond border (req {}, border {})",
                pos,
                self.size()
            );
            return;
        }

        // debug_assert!(pos.x < self.size.x);
        // debug_assert!(pos.y < self.size.y);
        debug_assert!(text.len() < (u16::max_value() as usize));

        let mut offset: u16 = 0;

        for (idx, grapheme) in text.graphemes(true).enumerate() {
            let shift_x = (idx as u16) + offset;

            if pos.x + shift_x >= self.size.x {
                break;
            }

            if (self.size.x as usize) - ((pos.x + shift_x) as usize) < grapheme.width() {
                dbg!("early quit on wide char.");
                break;
            }

            let xy = pos + XY::new(shift_x as u16, 0);
            self[xy] = Cell::Begin {
                style,
                grapheme: grapheme.to_string(),
            };

            for _ in 0..grapheme.width() - 1 {
                offset += 1;
                let cont_shift_x = (idx as u16) + offset;
                let xy2 = pos + XY::new(cont_shift_x as u16, 0 as u16);

                self[xy2] = Cell::continuation();
            }
        }
    }

    fn clear(&mut self) {
        for idx in 0..self.cells.len() {
            self.cells[idx] = Cell::empty();
        }
    }
}
