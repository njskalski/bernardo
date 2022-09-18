use std::fmt::{Debug, Formatter};

use log::error;
use unicode_width::UnicodeWidthStr;

use crate::io::output::Output;
use crate::io::style::{TEXT_STYLE_WHITE_ON_BLACK, TextStyle};
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::SizeConstraint;

pub struct SubOutput<'a> {
    output: &'a mut dyn Output,
    frame: Rect,
}

impl<'a> SubOutput<'a> {
    pub fn new(output: &'a mut dyn Output, frame: Rect) -> Self {
        // TODO add tests if frame is fully contained in Output and write errors to logs if its not.

        debug_assert!(output.size_constraint().bigger_equal_than(frame.lower_right()),
                      "frame = {}, output.size_constraint() = {}",
                      frame, output.size_constraint());

        SubOutput { output, frame }
    }
}

impl Output for SubOutput<'_> {
    /*
    Here pos is in "local space" of widget drawing to this SubOutput, which generally assumes it can
    draw from (0,0) to widget.size()
    and self.frame() is on "parent space".
    So we compare for "drawing beyond border" against *size* of the frame, not it's position.
     */
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        let end_pos = pos + (text.width_cjk() as u16, 0);

        if cfg!(debug_assertions) {
            debug_assert!(end_pos.x <= self.frame.size.x,
                          "drawing outside (to the right) the sub-output: ({} to {}) of {}",
                          pos, end_pos, self.frame.size);
            debug_assert!(end_pos.y < self.frame.size.y,
                          "drawing outside (below) the sub-output: ({} to {}) of {}",
                          pos, end_pos, self.frame.size);
        } else {
            if !(end_pos.x <= self.frame.size.x && end_pos.y < self.frame.size.y) {
                error!("drawing outside the sub-output: ({} to {}) of {}",
                    pos, end_pos, self.frame.size);
            }
        }

        // TODO add grapheme cutting
        self.output.print_at(self.frame.pos + pos, style, text)
    }

    fn clear(&mut self) -> Result<(), std::io::Error> {
        // DO NOT clear the wider output, clear only your part.

        let style = TEXT_STYLE_WHITE_ON_BLACK;

        for x in 0..self.frame.size.x {
            for y in 0..self.frame.size.y {
                self.output
                    .print_at(self.frame.pos + XY::new(x, y), style, " ")
            }
        }
        Ok(())
    }

    fn size_constraint(&self) -> SizeConstraint {
        let parent_sc = self.output.size_constraint();

        SizeConstraint::simple(self.frame.size)
    }
}

impl Debug for SubOutput<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<SubOutput sc : {:?} of {:?} >", self.size_constraint(), self.output)
    }
}