use crate::io::output::Output;
use crate::io::style::{TextStyle, TextStyle_WhiteOnBlack};
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
        // TODO add grapheme cutting
        self.output.print_at(self.frame.pos + pos, style, text)
    }

    fn clear(&mut self) {
        // DO NOT clear the wider output, clear only your part.

        let style = TextStyle_WhiteOnBlack;

        for x in 0..self.frame.size.x {
            for y in 0..self.frame.size.y {
                self.output
                    .print_at(self.frame.pos + XY::new(x, y), style, " ")
            }
        }
    }

    fn size_constraint(&self) -> SizeConstraint {
        SizeConstraint::simple(self.frame.size)
    }
}
