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
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        let end_pos = pos + (text.width_cjk() as u16, 0);

        if cfg!(debug_assertions) {
            debug_assert!(pos < self.frame.lower_right() && end_pos <= self.frame.lower_right(),
                          "drawing outside the sub-output: ({} to {}) at {}",
                          pos, end_pos, self.frame.lower_right());
        } else {
            if !(pos < self.frame.lower_right() && end_pos <= self.frame.lower_right()) {
                error!("drawing outside the sub-output: ({} to {}) at {}",
                    pos, end_pos, self.frame.lower_right());
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
        SizeConstraint::simple(self.frame.size)
    }
}
