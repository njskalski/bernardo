use std::io::ErrorKind;

use log::debug;
use log::error;
use log::warn;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::output::Output;
use crate::io::style::{TextStyle, TextStyle_WhiteOnBlack};
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

// Over output (maybe I'll rename it as super output) is an output that is bigger than original,
// physical or in-memory display. All write operations targeting lines/columns beyond it's borders
// are to be silently disregarded.

pub struct OverOutput<'a> {
    output: Box<&'a mut dyn Output>,
    upper_left_offset: XY,
    bottom_right_margin: XY,
}

impl<'a> OverOutput<'a> {
    pub fn new(
        output: Box<&'a mut dyn Output>,
        upper_left_offset: XY,
        bottom_right_margin: XY,
    ) -> Self {
        debug!(
            "making overoutput {:?} {:?} {:?}",
            upper_left_offset,
            output.size(),
            bottom_right_margin
        );
        OverOutput {
            output,
            upper_left_offset,
            bottom_right_margin,
        }
    }
}

impl SizedXY for OverOutput<'_> {
    fn size(&self) -> XY {
        self.upper_left_offset + self.output.size() + self.bottom_right_margin
    }
}

impl Output for OverOutput<'_> {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        if text.width() > u16::MAX as usize {
            warn!("got text width that would overflow u16::MAX, not drawing.");
            return;
        }

        if pos.y < self.upper_left_offset.y {
            return;
        }
        if pos.y > self.upper_left_offset.y + self.output.size().y {
            return;
        }

        let mut x_offset: i32 = 0;
        for grapheme in text.graphemes(true).into_iter() {
            let x = 0 as i32 + pos.x as i32 - self.upper_left_offset.x as i32 + x_offset;
            if x < 0 {
                continue;
            }
            if x > u16::MAX as i32 {
                warn!("got grapheme x position that would overflow u16::MAX, not drawing.");
                continue;
            }

            let y = pos.y - self.upper_left_offset.y; // > 0, tested above and < u16::MAX since no addition.
            let local_pos = XY::new(x as u16, y);

            self.output.print_at(local_pos, style, grapheme);
        }
    }

    fn get_visible_rect(&self) -> Rect {
        Rect::new(self.upper_left_offset, self.output.size())
    }

    fn clear(&mut self) {
        self.output.clear()
    }
}
