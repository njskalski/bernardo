use std::default::Default;
use std::fmt::{Debug, Formatter};
use std::ops::Range;

use log::{debug, warn};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::buffer::Buffer;
use crate::io::buffer_output::buffer_output_cells_iter::BufferOutputCellsIter;
use crate::io::buffer_output::buffer_output_consistent_items_iter::BufferConsistentItemsIter;
use crate::io::buffer_output::buffer_output_lines_iter::BufferLinesIter;
use crate::io::buffer_output::buffer_output_styles_iter::BufferStyleIter;
use crate::io::buffer_output::horizontal_iter_item::ConsistentHorizontalItem;
use crate::io::cell::Cell;
use crate::io::output::Output;
use crate::io::style::TextStyle;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub type BufferOutput = Buffer<Cell>;

impl Output for BufferOutput {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        // if !self.size_constraint().strictly_bigger_than(pos) {
        if pos >= self.size() {
            warn!("early exit on drawing beyond border (req {}, border {:?})", pos, self.size());
            return;
        }

        debug_assert!(text.len() < (u16::max_value() as usize));

        let mut offset: u16 = 0;

        for (idx, grapheme) in text.graphemes(true).enumerate() {
            let shift_x = (idx as u16) + offset;

            let max_x = self.size().x;

            if pos.x + shift_x >= max_x {
                break;
            }

            if (max_x as usize) - ((pos.x + shift_x) as usize) < grapheme.width() {
                debug!("early quit on wide char.");
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

    fn clear(&mut self) -> Result<(), std::io::Error> {
        for idx in 0..self.cells().len() {
            self.cells_mut()[idx] = Cell::default();
        }
        Ok(())
    }

    fn visible_rect(&self) -> Rect {
        let res = Rect::from_zero(self.size());
        debug_assert!(res.lower_right() <= self.size());
        res
    }

    #[cfg(test)]
    fn emit_metadata(&mut self, _meta: crate::io::output::Metadata) {}
}

impl Debug for BufferOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[BufferOutput {}]", self.size())
    }
}

impl BufferOutput {
    pub fn items_of_style(&self, style: TextStyle) -> BufferStyleIter {
        BufferStyleIter::new(&self, style)
    }

    pub fn consistent_items_iter(&self) -> BufferConsistentItemsIter {
        BufferConsistentItemsIter::new(self)
    }

    pub fn cells_iter(&self) -> BufferOutputCellsIter {
        BufferOutputCellsIter::new(self)
    }

    pub fn lines_iter(&self) -> BufferLinesIter {
        BufferLinesIter::new(self)
    }

    pub fn get_line(&self, line_idx: u16) -> Option<String> {
        if line_idx >= self.size().y {
            return None;
        }

        let mut res = String::new();
        res.reserve(self.size().x as usize);

        for x in 0..self.size().x {
            let pos = XY::new(x, line_idx);
            let cell = &self[pos];
            match cell {
                Cell::Begin { style: _, grapheme } => {
                    res += grapheme;
                }
                Cell::Continuation => {}
            }
        }

        Some(res)
    }

    pub fn get_horizontal_piece(&self, x: Range<u16>, y: u16) -> Option<BufferConsistentItemsIter> {
        if x.end > self.size().x {
            return None;
        }
        if y >= self.size().y {
            return None;
        }

        Some(BufferConsistentItemsIter::new(&self).with_rect(Rect::new(XY::new(x.start, y), XY::new(x.end, y + 1))))
    }
}

impl ToString for BufferOutput {
    fn to_string(&self) -> String {
        let size = self.size();
        let mut wchujdlugistring = String::new();
        wchujdlugistring.reserve((size.x * size.y + 1) as usize);

        for x in 0..size.x {
            for y in 0..size.y {
                let pos = XY::new(x, y);
                let cell = &self[pos];
                match cell {
                    Cell::Begin { style: _, grapheme } => {
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
